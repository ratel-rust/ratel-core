{
  var DISPLAY_TIMEOUT = 150;

  var clipboard = null;
  var generateAST = null;
  var generateASTEstree = null;
  var transform = null;

  var mode = 0;
  var valueTimer = 0;
  var output = null;
  var minify = false;

  Module.onRuntimeInitialized = bindUpdate(function (a) {
    generateAST = Module.cwrap('generate_ast', 'string', ['string', 'number']);
    generateASTEstree = Module.cwrap('generate_ast_estree', 'string', ['string', 'number']);
    transform = Module.cwrap('transform', 'string', ['string', 'number']);

    if (Clipboard.isSupported()) {
      clipboard = new Clipboard('.btn');
      copy_output.disabled = false;
    }

    bindEvents();
  });

  function bindListener (id, fn) {
    var el = document.getElementById(id);
    if (!el) {
      throw new Error('Element "' + id + '" not found');
    }
    el.onclick = bindUpdate(fn);
  }

  function bindUpdate (fn) {
    return function (e) {
      fn(e);

      copy_output.style.display = mode > 0 ? 'block' : 'none';
      run_output.style.display = mode === 0 ? 'block' : 'none';

      var minifyElement = checkbox_0.parentElement.MaterialCheckbox;

      if (minify) {
        minifyElement.check();
      } else {
        minifyElement.uncheck();
      }

      for (var i = 0; i < 3; ++i) {
        var radioElement = document.getElementById('output_' + i);
        var mdlNode = radioElement.parentElement.MaterialRadio;
        if (mode === i) {
          mdlNode.check();
        } else {
          mdlNode.uncheck();
        }
      }

      onInput(source_input.value, true);
    };
  }

  function onInput (value, immediate) {
    clearTimeout(valueTimer);
    valueTimer = setTimeout(function () {
      if (mode === 0) {
        output = transform(value, minify);
      } else if (mode === 1) {
        output = generateAST(value, minify);
      } else if (mode === 2) {
        output = generateASTEstree(value, minify);
      }
      ast_output.textContent = output;
    }, immediate ? 0 : DISPLAY_TIMEOUT);
  }

  function bindEvents () {
    source_input.oninput = function (e) {
      try {
        onInput(e.target.value);
      } catch (e) {
        ast_output.textContent = e.message || e;
      }
    };

    run_output.onclick = function (e) {
      try {
        var out = eval(output);
        console.log(out);
      } catch (e) {
        console.error(e.stack);
      }
    };

    bindListener('checkbox_0', function (e) {
      minify = e.target.checked ? 1 : 0;
    });

    bindListener('output_0', function (e) {
      mode = 0;
    });

    bindListener('output_1', function (e) {
      mode = 1;
    });

    bindListener('output_2', function (e) {
      mode = 2;
    });

    if (clipboard) {
      clipboard.on('success', function (e) {
        e.clearSelection();
      });

      clipboard.on('error', function (e) {
        throw e;
      });
    }
  }
}
