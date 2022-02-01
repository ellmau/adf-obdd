#!/usr/bin/env nix-shell
#! nix-shell -i zsh -p zsh

rm output.txt
rm times.txt
for i in /home/ellmau/repo/adf-obdd/res/adf-instances/instances/*.adf; do
    echo $i >> output.txt
    #({ time timeout 10s /home/ellmau/repo/adf-obdd/target/release/adf_bdd  --grd -q --lib hybrid $i } >> output.txt 2>&1 )
    #({ time timeout 10s /home/ellmau/repo/adf-obdd/target/release/adf_bdd  --com -q --lib hybrid $i } >> output.txt 2>&1 )
    #({ time timeout 10s /home/ellmau/repo/adf-obdd/target/release/adf_bdd  --stm -q --lib hybrid $i } >> output.txt 2>&1 )
    #({ time timeout 60s /home/ellmau/repo/adf-obdd/target/release/adf_bdd  --stmpre -q --lib hybrid $i } >> output.txt 2>&1 )
    #({ time timeout 10s clingo  0 /home/ellmau/scratch/diamond-adf-code/lib/ac2cico.lp /home/ellmau/scratch/diamond-adf-code/lib/show.lp /home/ellmau/scratch/diamond-adf-code/lib/base.lp /home/ellmau/scratch/diamond-adf-code/lib/opsm.lp /home/ellmau/scratch/diamond-adf-code/lib/cf.lp /home/ellmau/scratch/diamond-adf-code/lib/model.lp /home/ellmau/scratch/diamond-adf-code/lib/3KK.lp $i } >> output.txt 2>&1 )    
    #time timeout 10s /home/ellmau/repo/adf-obdd/target/release/adf_bdd  --com -q --lib hybrid $i >> output.txt 2>> times.txt
    #time timeout 10s /home/ellmau/repo/adf-obdd/target/release/adf_bdd  --stm -q --lib hybrid $i >> output.txt 2>> times.txt
    #time timeout 10s /home/ellmau/repo/adf-obdd/target/release/adf_bdd  --stmpre -q --lib hybrid $i >> output.txt 2>> times.txt
done

