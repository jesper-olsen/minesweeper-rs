# Set the directory where your data files are located
DATA_DIR="SolverDat"

# Loop through all files in the data directory
for DATA_FILE in "$DATA_DIR"/heatmap_*.txt; do
    # Get just the filename (e.g., "heatmap_beginner_unprotected") without the extension
    BASENAME=$(basename "${DATA_FILE%.dat}")

    # Initialize dimensions to default values
    width=0
    height=0

    # Determine dimensions based on the filename
    if [[ "$BASENAME" == *"beginner"* ]]; then
        width=9
        height=9
    elif [[ "$BASENAME" == *"intermediate"* ]]; then
        width=16
        height=16
    elif [[ "$BASENAME" == *"expert"* ]]; then
        width=30
        height=16
    else
        echo "Warning: Could not determine dimensions for $DATA_FILE. Skipping."
        continue # Skip to the next file
    fi

    # Create the output filename (e.g., "SolverDat/heatmap_beginner_unprotected.png")
    OUTPUT_FILE="${DATA_FILE%.txt}.png"

    echo "Processing $DATA_FILE with dimensions ${width}x${height}..."

    # Run the gnuplot script with all variables
    gnuplot -e "input_file='${DATA_FILE}'; output_file='${OUTPUT_FILE}'; width=${width}; height=${height}" plot_heatmap.gp
done
