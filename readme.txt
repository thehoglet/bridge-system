markdown
========
https://www.markdownguide.org/basic-syntax/
https://docs.newrelic.com/docs/style-guide/structure/styleguide-markup-indentation/

toml
====
https://docs.rs/toml/latest/toml/
https://quickref.me/toml.html

rust
====
https://stackoverflow.com/questions/27582739/how-do-i-create-a-hashmap-literal
https://stackoverflow.com/questions/72690767/how-to-parse-toml-in-rust-with-unknown-structure
https://rust-lang-nursery.github.io/rust-cookbook/encoding/complex.html#deserialize-a-toml-configuration-file
https://users.rust-lang.org/t/how-to-deserialize-nested-toml-structures-into-custom-structs/65920/2
https://gist.github.com/leonardo-m/6e9315a57fe9caa893472c2935e9d589
https://doc.rust-lang.org/core/iter/trait.Iterator.html
https://doc.rust-lang.org/std/io/enum.ErrorKind.html
https://www.lurklurk.org/effective-rust/cover.html
https://rust-lang.github.io/rustfmt/?version=v1.6.0&search=
https://stackoverflow.com/questions/66827800/how-to-throw-error-into-result-t-e-without-match-in-rust

windows
=======
https://stackoverflow.com/questions/2787203/unc-path-to-a-folder-on-my-local-computer

VBA
===

Sub MergeCells()

    Dim cel As Range
    Dim c As Range
    Dim selectedRange As Range
    Set selectedRange = Application.Selection
    
    Dim s As String
    s = ""

    For Each c In selectedRange.Cells
        If Len(s) > 0 Then
            s = s & " "
        End If
        s = s & Trim(c.Text)
        c.FormulaR1C1 = ""
    Next c
    Set c = selectedRange.Cells(1, 1)
    c.FormulaR1C1 = s

    Selection.Merge
    With Selection
        .HorizontalAlignment = xlGeneral
        .VerticalAlignment = xlTop
        .WrapText = True
        .MergeCells = True
    End With
    
    With Selection.Font
        .ColorIndex = xlAutomatic
    End With
    
End Sub
