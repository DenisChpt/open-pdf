use image::GenericImageView;
use lopdf::{dictionary, Document, Object, ObjectId, Stream};
use std::collections::BTreeMap;

use crate::domain::entities::{OverlayContent, OverlayElement, PageRange};
use crate::domain::errors::DomainError;
use crate::domain::ports::PdfProcessor;

/// Concrete adapter implementing PDF operations using the `lopdf` crate.
pub struct LopdfAdapter;

impl LopdfAdapter {
    pub fn new() -> Self {
        Self
    }

    fn load_document(pdf_data: &[u8]) -> Result<Document, DomainError> {
        Document::load_mem(pdf_data).map_err(|e| DomainError::InvalidPdf {
            reason: e.to_string(),
        })
    }


    fn save_document(doc: &mut Document) -> Result<Vec<u8>, DomainError> {
        let mut buf = Vec::new();
        doc.save_to(&mut buf).map_err(|e| DomainError::ProcessingError {
            reason: format!("Failed to serialize PDF: {}", e),
        })?;
        Ok(buf)
    }

    /// Recursively updates all object references within an object using the provided ID mapping.
    fn renumber_object_refs(object: &mut Object, mapping: &BTreeMap<ObjectId, ObjectId>) {
        match object {
            Object::Reference(id) => {
                if let Some(new_id) = mapping.get(id) {
                    *id = *new_id;
                }
            }
            Object::Dictionary(dict) => {
                for (_, value) in dict.iter_mut() {
                    Self::renumber_object_refs(value, mapping);
                }
            }
            Object::Array(arr) => {
                for item in arr.iter_mut() {
                    Self::renumber_object_refs(item, mapping);
                }
            }
            Object::Stream(stream) => {
                for (_, value) in stream.dict.iter_mut() {
                    Self::renumber_object_refs(value, mapping);
                }
            }
            _ => {}
        }
    }

    /// Finds the root Pages dictionary ObjectId from a document's catalog.
    fn root_pages_id(doc: &Document) -> Result<ObjectId, DomainError> {
        let catalog = doc.catalog().map_err(|e| DomainError::ProcessingError {
            reason: format!("Cannot read catalog: {}", e),
        })?;
        catalog
            .get(b"Pages")
            .and_then(|p| p.as_reference())
            .map_err(|e| DomainError::ProcessingError {
                reason: format!("Cannot read Pages reference: {}", e),
            })
    }
}

impl PdfProcessor for LopdfAdapter {
    fn page_count(&self, pdf_data: &[u8]) -> Result<u32, DomainError> {
        let doc = Self::load_document(pdf_data)?;
        Ok(doc.get_pages().len() as u32)
    }

    fn merge(&self, pdfs: Vec<Vec<u8>>) -> Result<Vec<u8>, DomainError> {
        if pdfs.len() < 2 {
            return Err(DomainError::InsufficientDocumentsForMerge);
        }

        // Start from the first document as base
        let mut base = Self::load_document(&pdfs[0])?;
        let base_pages_id = Self::root_pages_id(&base)?;

        for pdf_data in pdfs.iter().skip(1) {
            let doc = Self::load_document(pdf_data)?;

            // Offset all object IDs from this doc so they don't collide with base
            let offset = base.max_id;
            let mut id_mapping: BTreeMap<ObjectId, ObjectId> = BTreeMap::new();
            for &(old_id, generation) in doc.objects.keys() {
                id_mapping.insert((old_id, generation), (old_id + offset, generation));
            }

            // Copy all objects with remapped IDs
            for (old_id, object) in &doc.objects {
                let new_id = id_mapping[old_id];
                let mut new_object = object.clone();
                Self::renumber_object_refs(&mut new_object, &id_mapping);
                base.objects.insert(new_id, new_object);
            }

            // Update base.max_id
            base.max_id = base
                .objects
                .keys()
                .map(|&(id, _)| id)
                .max()
                .unwrap_or(base.max_id);

            // Get the source document's pages in order
            let pages = doc.get_pages();
            let mut sorted_pages: Vec<(u32, ObjectId)> = pages.into_iter().collect();
            sorted_pages.sort_by_key(|(num, _)| *num);

            // For each page, update its Parent to point to base's root Pages node,
            // then append it to the Kids array and increment Count.
            for (_, page_obj_id) in &sorted_pages {
                let new_page_id = id_mapping[page_obj_id];

                // Set Parent on the page
                if let Some(Object::Dictionary(page_dict)) = base.objects.get_mut(&new_page_id) {
                    page_dict.set("Parent", Object::Reference(base_pages_id));
                }

                // Append to Kids and increment Count on the root Pages node
                if let Some(Object::Dictionary(pages_dict)) = base.objects.get_mut(&base_pages_id) {
                    if let Ok(Object::Array(kids)) = pages_dict.get_mut(b"Kids") {
                        kids.push(Object::Reference(new_page_id));
                    }
                    if let Ok(count) = pages_dict.get(b"Count").and_then(Object::as_i64) {
                        pages_dict.set("Count", (count + 1) as u32);
                    }
                }
            }
        }

        // Clean up and renumber for a compact output
        base.prune_objects();
        base.renumber_objects();
        base.compress();

        Self::save_document(&mut base)
    }

    fn split(&self, pdf_data: &[u8], range: &PageRange) -> Result<Vec<u8>, DomainError> {
        let mut doc = Self::load_document(pdf_data)?;
        let pages = doc.get_pages();
        let total = pages.len() as u32;

        let pages_to_delete: Vec<u32> = (1..=total)
            .filter(|p| *p < range.start || *p > range.end)
            .collect();

        doc.delete_pages(&pages_to_delete);
        doc.prune_objects();
        doc.renumber_objects();

        Self::save_document(&mut doc)
    }

    fn compress(&self, pdf_data: &[u8]) -> Result<Vec<u8>, DomainError> {
        let mut doc = Self::load_document(pdf_data)?;

        // Decompress first to normalize, then recompress for optimal output
        doc.decompress();
        doc.delete_zero_length_streams();
        doc.prune_objects();
        doc.compress();
        doc.renumber_objects();

        Self::save_document(&mut doc)
    }

    fn reorder_pages(
        &self,
        pdf_data: &[u8],
        new_order: &[u32],
        rotations: &std::collections::HashMap<u32, u32>,
    ) -> Result<Vec<u8>, DomainError> {
        let mut doc = Self::load_document(pdf_data)?;
        let pages = doc.get_pages();
        let total = pages.len() as u32;
        let root_pages_id = Self::root_pages_id(&doc)?;

        // Validate page numbers
        for &page_num in new_order {
            if page_num == 0 || page_num > total {
                return Err(DomainError::PageOutOfRange {
                    page: page_num,
                    total,
                });
            }
        }

        // Build new Kids array in the desired order
        let mut kids = Vec::with_capacity(new_order.len());
        for &page_num in new_order {
            let page_id = pages[&page_num];

            if let Some(Object::Dictionary(dict)) = doc.objects.get_mut(&page_id) {
                dict.set("Parent", Object::Reference(root_pages_id));

                // Apply rotation if specified
                if let Some(&angle) = rotations.get(&page_num) {
                    let current: i64 = dict
                        .get(b"Rotate")
                        .and_then(|r| r.as_i64())
                        .unwrap_or(0);
                    let final_angle = ((current + angle as i64) % 360 + 360) % 360;
                    dict.set("Rotate", Object::Integer(final_angle));
                }
            }

            kids.push(Object::Reference(page_id));
        }

        // Update root Pages node
        if let Some(Object::Dictionary(pages_dict)) = doc.objects.get_mut(&root_pages_id) {
            pages_dict.set("Kids", Object::Array(kids));
            pages_dict.set("Count", new_order.len() as u32);
        }

        doc.prune_objects();
        doc.renumber_objects();

        Self::save_document(&mut doc)
    }

    fn split_by_size(
        &self,
        pdf_data: &[u8],
        max_bytes: u64,
        apply_compression: bool,
    ) -> Result<Vec<Vec<u8>>, DomainError> {
        let total = self.page_count(pdf_data)?;
        let mut result: Vec<Vec<u8>> = Vec::new();
        let mut start = 1u32;

        while start <= total {
            let mut best_end = start;
            let mut best_data: Vec<u8> = Vec::new();

            for end in start..=total {
                let range = PageRange::new(start, end, total)?;
                let mut data = self.split(pdf_data, &range)?;
                if apply_compression {
                    data = self.compress(&data)?;
                }

                if data.len() as u64 > max_bytes && end > start {
                    break;
                }

                best_data = data;
                best_end = end;

                if best_data.len() as u64 > max_bytes {
                    break;
                }
            }

            result.push(best_data);
            start = best_end + 1;
        }

        Ok(result)
    }

    fn overlay_elements(
        &self,
        pdf_data: &[u8],
        page_number: u32,
        elements: &[OverlayElement],
    ) -> Result<Vec<u8>, DomainError> {
        let mut doc = Self::load_document(pdf_data)?;
        let pages = doc.get_pages();
        let page_id = *pages.get(&page_number).ok_or(DomainError::PageOutOfRange {
            page: page_number,
            total: pages.len() as u32,
        })?;

        let (page_w, page_h) = Self::get_page_dimensions(&doc, page_id)?;

        // ── Phase 1: build all overlay objects ──

        // Font for text elements
        let font_id = doc.add_object(Object::Dictionary(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
            "Encoding" => "WinAnsiEncoding",
        }));

        let mut form_ops = String::new();
        let mut xobject_dict = lopdf::Dictionary::new();

        for (i, element) in elements.iter().enumerate() {
            let x = element.x_ratio * page_w;
            let y = page_h - (element.y_ratio * page_h) - (element.height_ratio * page_h);
            let w = element.width_ratio * page_w;
            let h = element.height_ratio * page_h;

            match &element.content {
                OverlayContent::Text { text, font_size } => {
                    let baseline_y = y + h * 0.25;
                    form_ops.push_str(&format!(
                        "q BT /SigFont {} Tf {} {} Td ({}) Tj ET Q\n",
                        font_size, x, baseline_y,
                        Self::escape_pdf_string(text),
                    ));
                }
                OverlayContent::Image { data } => {
                    let img = image::load_from_memory(data).map_err(|e| {
                        DomainError::ProcessingError {
                            reason: format!("Failed to decode image: {}", e),
                        }
                    })?;
                    let (img_w, img_h) = img.dimensions();

                    let has_alpha = img.color().has_alpha();
                    let img_stream_id = if has_alpha {
                        let rgba = img.to_rgba8();
                        let mut rgb_data = Vec::with_capacity((img_w * img_h * 3) as usize);
                        let mut alpha_data = Vec::with_capacity((img_w * img_h) as usize);
                        for pixel in rgba.pixels() {
                            rgb_data.extend_from_slice(&pixel.0[0..3]);
                            alpha_data.push(pixel.0[3]);
                        }

                        let mut mask_stream = Stream::new(
                            dictionary! {
                                "Type" => "XObject", "Subtype" => "Image",
                                "Width" => img_w as i64, "Height" => img_h as i64,
                                "ColorSpace" => "DeviceGray", "BitsPerComponent" => 8_i64,
                            },
                            alpha_data,
                        );
                        let _ = mask_stream.compress();
                        let mask_id = doc.add_object(Object::Stream(mask_stream));

                        let mut img_s = Stream::new(
                            dictionary! {
                                "Type" => "XObject", "Subtype" => "Image",
                                "Width" => img_w as i64, "Height" => img_h as i64,
                                "ColorSpace" => "DeviceRGB", "BitsPerComponent" => 8_i64,
                                "SMask" => Object::Reference(mask_id),
                            },
                            rgb_data,
                        );
                        let _ = img_s.compress();
                        doc.add_object(Object::Stream(img_s))
                    } else {
                        let rgb = img.to_rgb8();
                        let mut img_s = Stream::new(
                            dictionary! {
                                "Type" => "XObject", "Subtype" => "Image",
                                "Width" => img_w as i64, "Height" => img_h as i64,
                                "ColorSpace" => "DeviceRGB", "BitsPerComponent" => 8_i64,
                            },
                            rgb.into_raw(),
                        );
                        let _ = img_s.compress();
                        doc.add_object(Object::Stream(img_s))
                    };

                    let name = format!("SigImg{}", i);
                    xobject_dict.set(name.as_bytes(), Object::Reference(img_stream_id));

                    form_ops.push_str(&format!(
                        "q {} 0 0 {} {} {} cm /{} Do Q\n",
                        w, h, x, y, name,
                    ));
                }
            }
        }

        // ── Phase 2: create a self-contained Form XObject ──
        // A Form XObject carries its own Resources, so we never need to modify
        // the page's (potentially deeply-nested indirect) Resources at all.

        let mut form_resources = lopdf::Dictionary::new();
        let mut font_dict = lopdf::Dictionary::new();
        font_dict.set("SigFont", Object::Reference(font_id));
        form_resources.set("Font", Object::Dictionary(font_dict));
        if !xobject_dict.is_empty() {
            form_resources.set("XObject", Object::Dictionary(xobject_dict));
        }

        let form_stream = Stream::new(
            dictionary! {
                "Type" => "XObject",
                "Subtype" => "Form",
                "BBox" => vec![
                    Object::Integer(0), Object::Integer(0),
                    Object::Real(page_w), Object::Real(page_h),
                ],
                "Resources" => Object::Dictionary(form_resources),
            },
            form_ops.into_bytes(),
        );
        let form_id = doc.add_object(Object::Stream(form_stream));

        // ── Phase 3: add a tiny content stream that invokes the form ──
        let invoke_ops = format!("q /SigOverlay Do Q\n");
        let invoke_stream = Stream::new(dictionary! {}, invoke_ops.into_bytes());
        let invoke_id = doc.add_object(Object::Stream(invoke_stream));

        // ── Phase 4: register the form in the page's XObject resources ──
        // We only add ONE entry (/SigOverlay) — much simpler than adding fonts+images.
        Self::add_page_xobject(&mut doc, page_id, "SigOverlay", form_id)?;

        // ── Phase 5: append invoke stream to page Contents ──
        if let Some(Object::Dictionary(page_dict)) = doc.objects.get_mut(&page_id) {
            let existing = page_dict.get(b"Contents").ok().cloned();
            let mut arr = match existing {
                Some(Object::Reference(id)) => vec![Object::Reference(id)],
                Some(Object::Array(a)) => a,
                _ => vec![],
            };
            arr.push(Object::Reference(invoke_id));
            page_dict.set("Contents", Object::Array(arr));
        }

        Self::save_document(&mut doc)
    }
}

impl LopdfAdapter {
    /// Returns (width, height) in PDF points for the given page.
    fn get_page_dimensions(doc: &Document, page_id: ObjectId) -> Result<(f32, f32), DomainError> {
        let page_obj = doc.get_object(page_id).map_err(|e| DomainError::ProcessingError {
            reason: format!("Cannot read page: {}", e),
        })?;

        if let Object::Dictionary(dict) = page_obj {
            if let Ok(mb) = dict.get(b"MediaBox") {
                if let Ok(arr) = mb.as_array() {
                    if arr.len() >= 4 {
                        let w = Self::obj_to_f32(&arr[2]).unwrap_or(595.0);
                        let h = Self::obj_to_f32(&arr[3]).unwrap_or(842.0);
                        return Ok((w, h));
                    }
                }
            }
        }

        // Default to A4
        Ok((595.0, 842.0))
    }

    fn obj_to_f32(obj: &Object) -> Option<f32> {
        match obj {
            Object::Integer(i) => Some(*i as f32),
            Object::Real(f) => Some(*f as f32),
            _ => None,
        }
    }

    fn escape_pdf_string(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('(', "\\(")
            .replace(')', "\\)")
    }

    /// Adds an XObject entry to a page's Resources, handling indirect references at every level.
    fn add_page_xobject(
        doc: &mut Document,
        page_id: ObjectId,
        name: &str,
        xobject_id: ObjectId,
    ) -> Result<(), DomainError> {
        // Step 1: find the Resources object ID (or create inline)
        let resources_ref = {
            let page = doc.get_object(page_id)
                .and_then(|o| o.as_dict().map_err(|e| e.into()))
                .map_err(|_: lopdf::Error| DomainError::ProcessingError {
                    reason: "Cannot read page".into(),
                })?;
            match page.get(b"Resources") {
                Ok(Object::Reference(id)) => Some(*id),
                _ => None,
            }
        };

        let resources_id = if let Some(id) = resources_ref {
            id
        } else {
            // Ensure inline Resources dict exists on the page
            if let Some(Object::Dictionary(page_dict)) = doc.objects.get_mut(&page_id) {
                if page_dict.get(b"Resources").is_err() {
                    page_dict.set("Resources", Object::Dictionary(lopdf::Dictionary::new()));
                }
            }
            // Convert inline Resources to an indirect object for easier manipulation
            let res_dict = if let Some(Object::Dictionary(page_dict)) = doc.objects.get_mut(&page_id) {
                if let Ok(Object::Dictionary(d)) = page_dict.get(b"Resources") {
                    d.clone()
                } else {
                    lopdf::Dictionary::new()
                }
            } else {
                lopdf::Dictionary::new()
            };
            let id = doc.add_object(Object::Dictionary(res_dict));
            if let Some(Object::Dictionary(page_dict)) = doc.objects.get_mut(&page_id) {
                page_dict.set("Resources", Object::Reference(id));
            }
            id
        };

        // Step 2: find the XObject sub-dict (may also be indirect)
        let xobject_ref = {
            if let Some(Object::Dictionary(resources)) = doc.objects.get(&resources_id) {
                match resources.get(b"XObject") {
                    Ok(Object::Reference(id)) => Some(*id),
                    _ => None,
                }
            } else {
                None
            }
        };

        if let Some(xobj_id) = xobject_ref {
            // XObject dict is indirect — modify it directly
            if let Some(Object::Dictionary(xobjects)) = doc.objects.get_mut(&xobj_id) {
                xobjects.set(name, Object::Reference(xobject_id));
            }
        } else {
            // XObject is inline or missing — modify the Resources dict
            if let Some(Object::Dictionary(resources)) = doc.objects.get_mut(&resources_id) {
                if resources.get(b"XObject").is_err() {
                    resources.set("XObject", Object::Dictionary(lopdf::Dictionary::new()));
                }
                if let Ok(Object::Dictionary(xobjects)) = resources.get_mut(b"XObject") {
                    xobjects.set(name, Object::Reference(xobject_id));
                }
            }
        }

        Ok(())
    }
}
