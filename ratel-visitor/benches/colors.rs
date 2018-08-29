#![feature(test)]

extern crate test;
extern crate ratel;
extern crate ratel_visitor;

use ratel_visitor::{Visitable, Visitor};
use test::{Bencher, black_box};

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

struct DummyStaticVisitor;

impl<'ast> Visitor<'ast> for DummyStaticVisitor {}

#[bench]
fn empty_traverse(b: &mut Bencher) {
    let module = ratel::parse(SOURCE).expect("Must parse");
    let arena = module.arena();
    let offset = unsafe { arena.offset() };

    b.iter(|| {
        unsafe { arena.reset_to(offset) };

        black_box(module.visit_with(&mut DummyStaticVisitor));
    });
}
