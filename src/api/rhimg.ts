import { convertFileSrc } from "@tauri-apps/api/core";

function toBase64UrlNoPad(input: string): string {
  const bytes = new TextEncoder().encode(input);
  let binary = "";
  for (let i = 0; i < bytes.byteLength; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary)
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/, "");
}

export function rhimgUrl(route: string): string {
  return convertFileSrc(route, "rhimg");
}

export function thumbUrl(remoteUrl: string): string {
  const encoded = toBase64UrlNoPad(remoteUrl);
  return rhimgUrl(`thumb/${encoded}`);
}

export function pageUrl(
  gid: number,
  token: string,
  index: number,
  v?: number,
): string {
  const base = rhimgUrl(`page/${gid}/${token}/${index}`);
  return v !== undefined ? `${base}?v=${v}` : base;
}

export function localPageUrl(id: string, index: number): string {
  return rhimgUrl(`local/${id}/${index}`);
}
