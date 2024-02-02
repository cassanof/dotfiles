import argparse
import pandas as pd
from reportlab.lib import colors
from reportlab.lib.pagesizes import letter, landscape
from reportlab.platypus import SimpleDocTemplate, Table, TableStyle


parser = argparse.ArgumentParser(description='Convert CSV to PDF')
parser.add_argument('csv_file', help='The CSV file to convert')
parser.add_argument('pdf_file', help='The PDF file to create')
args = parser.parse_args()

csv_file = args.csv_file
pdf_file = args.pdf_file
df = pd.read_csv(csv_file)
df.fillna(" " * 10, inplace=True)


pdf = SimpleDocTemplate(
    pdf_file,
    pagesize=landscape(letter)
)
width, height = landscape(letter)

# Prepare data for the table (including the column names)
data = [df.columns.to_list()] + df.values.tolist()

# Create a table with the data
table = Table(data, colWidths=(width - 100) / len(df.columns))

# Add style to the table (grid, header color, etc.)
style = TableStyle([
    ('BACKGROUND', (0, 0), (-1, 0), colors.gray),
    ('TEXTCOLOR', (0, 0), (-1, 0), colors.whitesmoke),
    ('ALIGN', (0, 0), (-1, -1), 'CENTER'),
    ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
    ('BOTTOMPADDING', (0, 0), (-1, 0), 12),
    ('BACKGROUND', (0, 1), (-1, -1), colors.beige),
    ('GRID', (0, 0), (-1, -1), 1, colors.black)
])

table.setStyle(style)

# Add the table to the PDF
elems = []
elems.append(table)

pdf.build(elems)
