# This script takes care of packaging the build artifacts that will go in the
# release zipfile

$SRC_DIR = $PWD.Path
$STAGE = [System.Guid]::NewGuid().ToString()

Set-Location $ENV:Temp
New-Item -Type Directory -Name $STAGE
Set-Location $STAGE

$ZIP = "$SRC_DIR\$($Env:CRATE_NAME)-$($Env:APPVEYOR_REPO_TAG_NAME)-$($Env:TARGET).zip"
$RELEASE_DIR = "$SRC_DIR\target\$($Env:TARGET)\release"

Get-ChildItem $RELEASE_DIR
Copy-Item "$RELEASE_DIR\gcode.dll" '.\'
Copy-Item "$RELEASE_DIR\gcode.dll.lib" '.\'
Copy-Item "$RELEASE_DIR\gcode.lib" '.\'

7z a "$ZIP" *

Push-AppveyorArtifact "$ZIP"

Remove-Item *.* -Force
Set-Location ..
Remove-Item $STAGE
Set-Location $SRC_DIR
