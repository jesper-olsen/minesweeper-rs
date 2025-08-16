# run with 
# gnuplot -e "input_file='heatmap.txt'; output_file='heatmap.png'; width=9; height=9" plot_heatmap.gp

# Use the width and height variables to set the terminal size and aspect ratio
set terminal pngcairo size 800,800*(1.0*height/width)
set size ratio -1

set output output_file

# Dynamically set the range based on the passed width and height
set xrange [-0.5:width-0.5]
set yrange [-0.5:height-0.5]

set pm3d map
set palette rgb 33,13,10

unset key

plot input_file matrix with image
