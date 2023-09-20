# entomon
debugging force field benchmark results

## Demo
In the demo below, you can see the generation of SVGs corresponding to QCArchive record IDs.

![demo](demo.gif)

## Queries
The server supports a basic query language for selecting subsets of the data.
Supported characters are given in the table below:

| Character | Meaning                                    |
|-----------|--------------------------------------------|
| &#124;    | Take the absolute value of the query field |
| $         | Field reference                            |
| >         | Greater than                               |
| <         | Less than                                  |
| >=        | Greater than or equal to                   |
| <=        | Less than or equal to                      |
| [0-9.-]   | Numbers                                    |

For example, the query `|$1|>5` will return the rows for which the first column
of data has a magnitude greater than 5.
