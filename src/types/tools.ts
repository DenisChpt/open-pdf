export interface Tool {
  id: string;
  name: string;
  description: string;
  icon: string;
  route: string;
  color: string;
}

export const TOOLS: Tool[] = [
  {
    id: "merge",
    name: "Fusionner PDF",
    description: "Combiner plusieurs fichiers PDF en un seul document",
    icon: "M4 6h16M4 12h16M4 18h16",
    route: "/merge",
    color: "#e74c3c",
  },
  {
    id: "split",
    name: "Diviser PDF",
    description: "Extraire des pages d'un fichier PDF",
    icon: "M7 21L17 3M5 21h14M5 3h14",
    route: "/split",
    color: "#3498db",
  },
  {
    id: "compress",
    name: "Compresser PDF",
    description: "Reduire la taille de vos fichiers PDF",
    icon: "M19 14l-7 7m0 0l-7-7m7 7V3",
    route: "/compress",
    color: "#27ae60",
  },
  {
    id: "organize",
    name: "Organiser PDF",
    description: "Reorganiser, supprimer ou deplacer les pages",
    icon: "M4 8V4m0 0h4M4 4l5 5M20 8V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5M20 16v4m0 0h-4m4 0l-5-5",
    route: "/organize",
    color: "#9b59b6",
  },
  {
    id: "convert",
    name: "PDF vers Image",
    description: "Convertir les pages de votre PDF en images PNG, JPG ou WEBP",
    icon: "M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z",
    route: "/convert",
    color: "#00b894",
  },
  {
    id: "sign",
    name: "Signer PDF",
    description: "Ajoutez votre signature, initiales, tampon ou date",
    icon: "M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z",
    route: "/sign",
    color: "#e67e22",
  },
];
