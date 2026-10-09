const FRIENDLY: Record<string, string> = {
  "HTTP request failed":
    "The network request failed. Check your connection and try again.",
  "I/O operation failed":
    "The disk could not be written. Free some space and try again.",
  "downloaded SHA1 does not match": "A downloaded file was damaged. Try again.",
};

export function messageOf(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "Something went wrong.";
}

export function friendlyError(message: string): string {
  return FRIENDLY[message] ?? message;
}
