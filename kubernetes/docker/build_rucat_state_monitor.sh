cp -r ../../rucat_state_monitor .
cp -r ../../rucat_common .

docker build -t rucat_state_monitor:0.1.0 -f rucat_state_monitor.dockerfile .

rm -rf rucat_common
rm -rf rucat_state_monitor