#![feature(test)]

extern crate test;
extern crate ratel;

use test::Bencher;

static SOURCE: &'static str = r#"

'use strict';

/**
 * Extract red color out of a color integer:
 *
 * 0x00DEAD -> 0x00
 *
 * @param  {Number} color
 * @return {Number}
 */
function red( color )
{
    let foo = 3.14;
    return color >> 16;
}

/**
 * Extract green out of a color integer:
 *
 * 0x00DEAD -> 0xDE
 *
 * @param  {Number} color
 * @return {Number}
 */
function green( color )
{
    return ( color >> 8 ) & 0xFF;
}


/**
 * Extract blue color out of a color integer:
 *
 * 0x00DEAD -> 0xAD
 *
 * @param  {Number} color
 * @return {Number}
 */
function blue( color )
{
    return color & 0xFF;
}


/**
 * Converts an integer containing a color such as 0x00DEAD to a hex
 * string, such as '#00DEAD';
 *
 * @param  {Number} int
 * @return {String}
 */
function intToHex( int )
{
    const mask = '#000000';

    const hex = int.toString( 16 );

    return mask.substring( 0, 7 - hex.length ) + hex;
}


/**
 * Converts a hex string containing a color such as '#00DEAD' to
 * an integer, such as 0x00DEAD;
 *
 * @param  {Number} num
 * @return {String}
 */
function hexToInt( hex )
{
    return parseInt( hex.substring( 1 ), 16 );
}

module.exports = {
    red,
    green,
    blue,
    intToHex,
    hexToInt,
};

"#;

#[bench]
fn parse_to_ast(b: &mut Bencher) {
    b.bytes = SOURCE.len() as u64;

    b.iter(|| {
        let _module = ratel::parser::parse(SOURCE).expect("Must parse");
    });
}


#[bench]
fn tokenize(b: &mut Bencher) {
    b.bytes = SOURCE.len() as u64;

    b.iter(|| {
        let mut lexer = ratel::lexer::Lexer::new(SOURCE);

        while lexer.get_token() != ratel::lexer::Token::EndOfProgram {

        }
    });
}

// #[bench]
// fn parse_to_ast_and_transform_es5(b: &mut Bencher) {
//     b.bytes = SOURCE.len() as u64;

//     b.iter(|| {
//         let mut ast = ratel::parser::parse(SOURCE.to_owned()).expect("Must parse");

//         let settings = ratel::transformer::Settings::target_es5();

//         ratel::transformer::transform(&mut ast, settings);

//         ast
//     });
// }

// #[bench]
// fn codegen_from_ast(b: &mut Bencher) {
//     let mut ast = ratel::parser::parse(SOURCE.to_owned()).expect("Must parse");

//     let settings = ratel::transformer::Settings::target_es5();

//     ratel::transformer::transform(&mut ast, settings);

//     let output = ratel::codegen::generate_code(&ast, true);

//     b.bytes = output.len() as u64;

//     b.iter(|| {
//         ratel::codegen::generate_code(&ast, true)
//     });
// }
