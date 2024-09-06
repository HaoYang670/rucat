cp -r ../../rucat_server .
cp -r ../../rucat_common .

docker build -t rucat_server:0.1.0 -f rucat_server.dockerfile .

rm -rf rucat_common
rm -rf rucat_server