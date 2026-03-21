import { open, save } from "@tauri-apps/plugin-dialog";

export function useFileDialog() {
  async function pickPdfFiles(): Promise<string[]> {
    const result = await open({
      multiple: true,
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });

    if (!result) return [];
    return Array.isArray(result) ? result : [result];
  }

  async function pickSavePath(defaultName: string): Promise<string | null> {
    const result = await save({
      defaultPath: defaultName,
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });

    return result;
  }

  async function pickDirectory(): Promise<string | null> {
    const result = await open({
      directory: true,
      multiple: false,
    });

    if (!result) return null;
    return Array.isArray(result) ? result[0] : result;
  }

  return { pickPdfFiles, pickSavePath, pickDirectory };
}
