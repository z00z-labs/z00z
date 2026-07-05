#!/bin/bash

# Code Refactoring Skill - File Size Checker
# Quick utility to check file sizes and provide refactoring recommendations

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Function to check single file
check_file() {
    local file=$1

    if [ ! -f "$file" ]; then
        echo -e "${RED}Error: File not found: $file${NC}"
        return 1
    fi

    # Get line count
    local lines=$(wc -l < "$file")
    local filename=$(basename "$file")

    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "File: $filename"
    echo "Lines: $lines"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    # Provide assessment based on file type and size
    if [[ $file == *.tsx ]] || [[ $file == *.jsx ]]; then
        # React components
        if [ $lines -lt 150 ]; then
            echo -e "${GREEN}✅ Status: Good${NC}"
            echo "Component size is within recommended limits."
        elif [ $lines -lt 200 ]; then
            echo -e "${YELLOW}⚠️  Status: Warning${NC}"
            echo "Component is getting large. Consider:"
            echo "  • Extract data to separate file"
            echo "  • Move modal/dialog to separate component"
            echo "  • Extract custom hooks if 4+ hooks used"
        elif [ $lines -lt 300 ]; then
            echo -e "${YELLOW}🚨 Status: Alert${NC}"
            echo "Component should be refactored before adding more."
            echo "Recommendations:"
            echo "  • Split into sub-components"
            echo "  • Extract data files"
            echo "  • Create custom hooks"
        else
            echo -e "${RED}🛑 Status: Critical${NC}"
            echo "Component MUST be refactored immediately!"
            echo "Action required:"
            echo "  • Create feature directory structure"
            echo "  • Split into multiple components"
            echo "  • Extract all data and hooks"
        fi
    elif [[ $file == *.py ]]; then
        # Python files
        if [ $lines -lt 250 ]; then
            echo -e "${GREEN}✅ Status: Good${NC}"
            echo "Python file size is within recommended limits."
        elif [ $lines -lt 400 ]; then
            echo -e "${YELLOW}⚠️  Status: Warning${NC}"
            echo "File is getting large. Consider:"
            echo "  • Extract configuration to separate file"
            echo "  • Split large classes"
            echo "  • Break down long functions (>50 lines)"
        else
            echo -e "${RED}🛑 Status: Critical${NC}"
            echo "File should be refactored!"
            echo "Recommendations:"
            echo "  • Convert module to package"
            echo "  • Split classes by responsibility"
            echo "  • Extract utilities to separate modules"
        fi
    else
        # General files
        if [ $lines -lt 200 ]; then
            echo -e "${GREEN}✅ Status: Good${NC}"
            echo "File size is reasonable."
        elif [ $lines -lt 300 ]; then
            echo -e "${YELLOW}⚠️  Status: Warning${NC}"
            echo "File is getting large. Consider refactoring."
        else
            echo -e "${RED}🛑 Status: Critical${NC}"
            echo "File should be refactored to improve maintainability."
        fi
    fi

    echo ""
}

# Function to check directory
check_directory() {
    local dir=$1
    local pattern=$2

    if [ ! -d "$dir" ]; then
        echo -e "${RED}Error: Directory not found: $dir${NC}"
        return 1
    fi

    echo ""
    echo "Scanning directory: $dir"
    echo "Pattern: $pattern"
    echo ""

    local large_files=0
    local warning_files=0
    local total_files=0

    # Find files and check each one
    while IFS= read -r file; do
        ((total_files++))
        local lines=$(wc -l < "$file")
        local filename=$(basename "$file")

        if [ $lines -gt 300 ]; then
            echo -e "${RED}🛑 $filename: $lines lines (CRITICAL)${NC}"
            ((large_files++))
        elif [ $lines -gt 200 ]; then
            echo -e "${YELLOW}🚨 $filename: $lines lines (ALERT)${NC}"
            ((warning_files++))
        elif [ $lines -gt 150 ]; then
            echo -e "${YELLOW}⚠️  $filename: $lines lines (WARNING)${NC}"
            ((warning_files++))
        fi
    done < <(find "$dir" -type f -name "$pattern")

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Summary for $dir"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Total files scanned: $total_files"
    echo "Files needing attention: $((large_files + warning_files))"
    echo "  - Critical (>300 lines): $large_files"
    echo "  - Warning (150-300 lines): $warning_files"
    echo ""
}

# Main script logic
main() {
    if [ $# -eq 0 ]; then
        echo "Usage:"
        echo "  $0 <file>              Check single file"
        echo "  $0 <directory> <pattern>   Check directory with pattern"
        echo ""
        echo "Examples:"
        echo "  $0 src/components/Dashboard.tsx"
        echo "  $0 src/components '*.tsx'"
        echo "  $0 src/services '*.py'"
        exit 1
    fi

    if [ -f "$1" ]; then
        # Single file check
        check_file "$1"
    elif [ -d "$1" ]; then
        # Directory check
        pattern=${2:-"*"}
        check_directory "$1" "$pattern"
    else
        echo -e "${RED}Error: Not a valid file or directory: $1${NC}"
        exit 1
    fi
}

# Run main function
main "$@"
