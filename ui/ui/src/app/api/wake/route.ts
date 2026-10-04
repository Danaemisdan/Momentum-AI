export async function POST() {
  return new Response(JSON.stringify({
    status: "offline",
    message: "Momentum runtime is managed by start.sh. Run the launcher from the project root.",
  }), {
    status: 409,
    headers: { "content-type": "application/json" },
  });
}
