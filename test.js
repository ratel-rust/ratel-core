let foo = 'lorem',
    bar = "ipsum";

const pi = 3.14

// 42
var binary = 0b101010,
    octal = 0o52,
    hexal = 0x2A;

let pojo = {
    foo,
    id: 9001,
    name: "Maciej",
    'is-radical': true,
    [foo + bar]: 'totally'
};


[1, 2, 3].forEach(n => n * n);

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

let foo = 1 + 2 * 3 - 5 ** 2;

var µDivs = µ( 'div' ) ;

let baz = true ? .5 : .25;

if (Math.random() > 0.5) {
    console.log("Maybe");
} else if (Math.random() > 0.5) {
    console.log("Much maybe?");
} else {
    console.log("Maybe not!");
}
