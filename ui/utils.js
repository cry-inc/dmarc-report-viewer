export function join(elements, joiner) {
    return elements.flatMap(x => [joiner, x]).slice(1);
}
