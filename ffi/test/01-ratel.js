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
      }, /Unexpected token/);
    });

    it('parses', () => {
      const result = Ratel.parse('2');
      assert.equal(typeof result, 'string');
      const expected = `[Loc { start: 0, end: 1, item: Expression { expression: Loc { start: 0, end: 1, item: Value(Number("2")) } } }]`;
      assert.equal(result, expected);
    });
  });

  describe('ast', () => {

    it('returns an AST', () => {
      const result = Ratel.ast('const double = function (a, b = 2) { return a * 2}');
      const json = JSON.parse(result, null, '  ');
      assert.deepEqual(json, {
        type: 'Program',
        end: 0,
        start: 0,
        body: [
          {
            type: 'VariableDeclaration',
            start: 0,
            end: 0,
            kind: 'const',
            declarations: [
              {
                type: 'VariableDeclarator',
                id: { end: 12, name: 'double', start: 6, type: 'Identifier' },
                start: 0,
                end: 0,
                init: {
                  type: 'FunctionExpression',
                  id: null,
                  start: 0,
                  end: 0,
                  params: [
                    {
                      type: 'Identifier',
                      name: 'a',
                      start: 26,
                      end: 27,
                    },
                    {
                      type: 'AssignmentPattern',
                      left: { end: 0, name: 'b', start: 0, type: 'Identifier' },
                      right: { end: 33, value: '2', start: 32, type: 'Literal' },
                      start: 33,
                      end: 34,
                    }
                  ],
                  body: {
                    type: "BlockStatement",
                    start: 0,
                    end: 0,
                    body: [
                      {
                        type: 'ReturnStatement',
                        end: 0,
                        start: 0,
                        argument: {
                          type: 'BinaryExpression',
                          start: 44,
                          end: 49,
                          left: {
                            type: 'Identifier',
                            name: 'a',
                            start: 44,
                            end: 45,
                          },
                          operator: '*',
                          right: {
                            type: 'Literal',
                            value: '2',
                            end: 49,
                            start: 48,
                          }
                        }
                      }
                    ]
                  }
                }
              }
            ]
          }
        ]
      });
    });
  });
});
