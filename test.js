let foo = 'lorem',
    bar = "ipsum";

const pi = 3.14

// 42
var binary = 0b101010,
    octal = 0o52,
    hexal = 0x2A;

let fifty = binary + 2 + 4 + 1 + 1;

let pojo = {
    id: 9001,
    name: "Maciej",
    'is-radical': true,
    [foo + bar]: 'totally'
};


[1, 2, 3].forEach(n => n * n)

['fooz', 'baz'].map(item => item.toUpperCase());

function helloKitty(count, name) {
    while (count--) console.log(name);

    return false;
}

helloKitty();

let emptyArray = [];

/*
    boring block comment
*/
class Foo {
    x = 0;
    y = 0;
    static isFoo = true;

    constructor() {
        console.log('New instance of Foo');
    }

    bar(n) {
        console.log('Called bar with ' + n);
    }

    static baz() {
        console.log('Static method baz!');
    }
}

++emptyArray;
