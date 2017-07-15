describe('Ratel ffi', () => {
  it('is an object, has transform and parse methods', () => {
    assert.equal(typeof Ratel, 'object');
    assert.equal(typeof Ratel.transform, 'function');
    assert.equal(typeof Ratel.parse, 'function');
  });

  describe('transform', () => {
    it('throws an error without a string as first argument', () => {
      assert.throws(() => {
        Ratel.transform();
      });
    });

    it('throws an error without a boolean as second argument', () => {
      assert.throws(() => {
        Ratel.transform('');
      });
    });

    it('transforms', () => {
      // const result = Ratel.transform('2**2', true);
      const result = Ratel.transform('Math.pow(2, 2)', true);
      assert.equal(typeof result, 'string');
      assert.equal(result, 'Math.pow(2,2);');
    });
  });

  describe('parse', () => {
    it('throws an error without a string as first argument', () => {
      assert.throws(() => {
        Ratel.parse();
      });
    });

    it('throws syntax errors', () => {
      assert.throws(() => {
        Ratel.parse('function function () {}');
      }, /UnexpectedToken/);
    });

    it('parses', () => {
      const result = Ratel.parse('2');
      assert.equal(typeof result, 'string');
      const expected = `[Loc { start: 0, end: 1, item: Expression { expression: Loc { start: 0, end: 1, item: Value(Number("2")) } } }]`;
      assert.equal(result, expected);
    });
  });
});
