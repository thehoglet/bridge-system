<#
.SYNOPSIS
Export each worksheet in target workbook as CSV
.PARAMETER sourceFile
The relative or fully qualified path to the workbook file
.PARAMETER destinationDirectory
The relative or fully qualified path to the file to which to write the CSV 
.EXAMPLE
C:\PS> Export-CsvFromXslx.ps1 .\example.xslx .\csv\
#>

[CmdletBinding()]

param (

  [Parameter(Mandatory = $true)]
  [ValidateScript( {
    if (-not ($_ | Test-Path -PathType Leaf))
    {
      throw "$_ does not exist"
    }
    return $true
  })]
  [System.IO.FileInfo]$sourceFile,

  [Parameter(Mandatory = $true)]
  [System.IO.DirectoryInfo]$destinationDirectory

)

if (!(Test-Path -Path $destinationDirectory -PathType Container)) {
  New-Item -Path $destinationDirectory -ItemType "directory" | Out-Null  
}

$excel = New-Object -Com Excel.Application

try {
  
  $wb = $excel.Workbooks.Open((Resolve-Path $sourceFile))
  $nWorksheets = $wb.Worksheets.count
  $xlCSVUTF8 = 62
  $targetRoot = Resolve-Path -Path $destinationDirectory
  
  for ($i = 1; $i -le $nWorksheets; ++$i) {
  
    $ws = $wb.Worksheets.Item($i)
    $destinationFile = Join-Path $targetRoot ($ws.Name + '.csv')

    if (Test-Path -Path $destinationFile -PathType Leaf) {
      Remove-Item -Path $destinationFile -Force
    }

    $ws.SaveAs($destinationFile, $xlCSVUTF8)  
  }
}
catch {
  Write-Host "An error occurred:"
  Write-Host $_
}

$excel.Quit()

exit 0
