export function unreachable(): never {
    throw new Error(
      "this statement is unreachable; this check exists to make TS happy"
    );
}

export function replaceElementByIdWithNewEmptyDiv(id: string) {
    const old_element = document.getElementById(id);
    if (old_element == null) {
        return;
    }
    const new_element = document.createElement("div");
    new_element.id = id;
    old_element.replaceWith(new_element);
    return new_element;
}