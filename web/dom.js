export function createElement(tag, className = "", text = "") {
  const element = document.createElement(tag);
  if (className) element.className = className;
  if (text) element.textContent = text;
  return element;
}

export function createButton(label, onClick, className = "") {
  const button = createElement("button", className, label);
  button.type = "button";
  button.addEventListener("click", onClick);
  return button;
}
