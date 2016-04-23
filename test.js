let foo = 'lorem',
    bar = "ipsum";
const pi = 3.14;

// 42
let binary = 0b101010;

// also 42
let octal = 0o52;

// yep, also 42
let hexal = 0x2A;

var pojo = {
    id: 9001,
    name: "Maciej",
    'is-radical': true,
    [foo + bar]: 'totally'
};

// let emptyArray = [];

var squares = [
    1,
    2,
    3,
    [
        'a',
        'b',
        'c',
    ],
];

//.forEach(n => n * n);


// /*
//     boring block comment
// */
// export default class Example {
//     isFooBar() {
//         if (foo === "bar") {
//             return true;
//         }

//         // random comment
//         return false;
//     }
// }

// export function maybe() {
//     return Math.random() > 0.5;
// }
