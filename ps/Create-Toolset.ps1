<#
.SYNOPSIS
Export each worksheet in target workbook as CSV
.PARAMETER sourceFile
The relative or fully qualified path to the workbook file
.PARAMETER destinationDirectory
The relative or fully qualified path to the file to which to write the CSV 
.EXAMPLE
C:\PS> Create-Toolset.ps1
#>

[CmdletBinding()]

param (

)

$destinationDirectory = Join-Path $PSScriptRoot "..\tools"

if (!(Test-Path -Path $destinationDirectory -PathType Container)) {
    New-Item -Path $destinationDirectory -ItemType "directory" | Out-Null  
}

Function Copy-Tool($sourceFileRelative) {

    $sourceFile = Join-Path $PSScriptRoot $sourceFileRelative

    if (-not (Test-Path -Path $sourceFile -PathType Leaf))
    {
        Write-Error -Message "file not found: $sourceFile"
        exit 1
    }
    
    Copy-Item -Path $sourceFile -Destination $destinationDirectory
}

Copy-Tool ".\Export-CsvFromXslx.ps1"
Copy-Tool "..\rs\target\release\csv-to-toml.exe"
Copy-Tool "..\rs\target\release\toml-to-md.exe"
