import type { DownloadProgress } from "ky";
import api from ".";

export async function downloadFile(
  url: string,
  searchParams?: { [key: string]: string },
  onDownloadProgress?: (progress: DownloadProgress) => void
) {
  return await api
    .get(url, {
      searchParams,
      onDownloadProgress,
    })
    .blob();
}
