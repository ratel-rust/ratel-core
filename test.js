let foo = 'bar';
const pi = 3.14;
var squares = [1, 2, 3].forEach(n => n * n);
let binary = 0b101010; // 42
let octal = 0o52;     // also 42
let hexal = 0x2A;     // yep, also 42

/*
    boring block comment
*/
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
