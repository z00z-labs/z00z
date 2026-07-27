export function slugifyHeading(heading) {
  if (typeof heading !== "string") throw new TypeError("Heading must be a string");

  return encodeURIComponent(
    heading
      .trim()
      .toLowerCase()
      .replace(/[^\p{Letter}\p{Number}\s-]+/gu, "")
      .replace(/\s+/gu, "-"),
  );
}
