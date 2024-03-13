export function unreachable(): never {
    throw new Error(
      "this statement is unreachable; this check exists to make TS happy"
    );
}