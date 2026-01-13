#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          GENERATING NSA-LEVEL SECURITY REPORT PDF             ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Check if pandoc is installed
if ! command -v pandoc &> /dev/null; then
    echo "Installing pandoc for PDF generation..."
    brew install pandoc
    brew install basictex
fi

# Generate timestamp
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Create enhanced markdown with better formatting
cat > NSA_ENHANCED_REPORT.md << 'EOF'
---
title: "NSA-Level Network Security Assessment"
subtitle: "Comprehensive Threat Detection Report"
author: "Security Analysis Division"
date: "January 3, 2026"
geometry: margin=1in
fontsize: 11pt
header-includes:
  - \usepackage{fancyhdr}
  - \pagestyle{fancy}
  - \fancyhead[L]{CLASSIFICATION: UNCLASSIFIED}
  - \fancyhead[R]{Network Security Report}
---

\newpage

EOF

# Append the main report
cat NSA_THREAT_DETECTION_REPORT.md >> NSA_ENHANCED_REPORT.md

# Generate PDF using pandoc
echo "Generating PDF report..."
pandoc NSA_ENHANCED_REPORT.md \
    -o "NSA_Security_Report_${TIMESTAMP}.pdf" \
    --pdf-engine=xelatex \
    --toc \
    --toc-depth=2 \
    --highlight-style=tango \
    2>/dev/null || {
    echo "Note: PDF generation requires LaTeX. Using HTML fallback..."
    pandoc NSA_ENHANCED_REPORT.md \
        -o "NSA_Security_Report_${TIMESTAMP}.html" \
        --standalone \
        --toc \
        --toc-depth=2 \
        --css=https://cdnjs.cloudflare.com/ajax/libs/github-markdown-css/5.2.0/github-markdown.min.css
    echo "HTML report generated: NSA_Security_Report_${TIMESTAMP}.html"
}

# Check if PDF was created
if [ -f "NSA_Security_Report_${TIMESTAMP}.pdf" ]; then
    echo ""
    echo "✅ PDF Report Generated Successfully!"
    echo "📄 File: NSA_Security_Report_${TIMESTAMP}.pdf"
    echo "📊 Size: $(du -h NSA_Security_Report_${TIMESTAMP}.pdf | cut -f1)"
    echo ""
    echo "Opening PDF..."
    open "NSA_Security_Report_${TIMESTAMP}.pdf" 2>/dev/null || echo "Please open the PDF manually"
else
    echo "⚠️  PDF generation failed. HTML version may be available."
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "                    REPORT GENERATION COMPLETE"
echo "════════════════════════════════════════════════════════════════"
