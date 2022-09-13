if not defined in_subprocess (cmd /k set in_subprocess=y ^& %0 %*) & exit )

cmd /K cd \\192.168.1.102\Windows\hello_egui>
cmd /k "cargo run"