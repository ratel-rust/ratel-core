echo "TESTING: ratel"
echo "##############"
echo ""

cd ratel
cargo test || exit
cd ..

echo ""
echo ""
echo "TESTING: ratel-codegen"
echo "######################"
echo ""

cd ratel-codegen
cargo test || exit
cd ..

echo ""
echo ""
echo "TESTING: ratel-visitor"
echo "######################"
echo ""

cd ratel-visitor
cargo test || exit
cd ..

echo ""
echo ""
echo "TESTING: ratel-transformer"
echo "##########################"
echo ""

cd ratel-transformer
cargo test || exit
cd ..

echo ""
echo ""
echo "TESTING: ffi"
echo "############"
echo ""

cd ffi
npm i || exit
npm test || exit
cd ..
