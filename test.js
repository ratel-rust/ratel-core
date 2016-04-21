let foo = 'bar';
const pi = 3.14;
var squares = [1, 2, 3].forEach(n => n * n);

/* boring block comment */
export default class Example {
    isFooBar() {
        if (foo === "bar") {
            return true;
        }

        // random comment
        return false;
    }
}

export function maybe() {
    return Math.random() > 0.5;
}
