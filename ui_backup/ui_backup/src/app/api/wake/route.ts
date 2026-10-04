// This API route is intentionally disabled in the Tauri static build.
// The Tauri shell (src-tauri/src/main.rs) manages all process lifecycle.
// This file exists only to prevent import errors during development.
export const dynamic = 'force-static';
export async function POST() {
  return new Response(JSON.stringify({ status: "managed_by_tauri" }), {
    status: 200,
    headers: { "content-type": "application/json" },
  });
}
