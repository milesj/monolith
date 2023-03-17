#!/usr/bin/env pwsh
# Copyright 2022 moonrepo, Inc.

# Thanks to Deno for the original script:
# https://github.com/denoland/deno_install/blob/master/install.ps1

$ErrorActionPreference = 'Stop'

$Version = "0.3.1" # TODO

if ($Args.Length -eq 1) {
  $Version = $Args.Get(0)
}

$Target = "proto_cli-v${Version}-x86_64-pc-windows-msvc"

$DownloadUrl = if ($Version -eq "latest") {
  "https://github.com/moonrepo/proto/releases/latest/download/${Target}.zip"
} else {
  "https://github.com/moonrepo/proto/releases/download/v${Version}/${Target}.zip"
}

$TempDir = "${HOME}\.proto\temp\proto\${Target}"
$DownloadFile = "${TempDir}.zip"
$InstallDir = "${Home}\.proto\bin"
$BinPath = "${InstallDir}\proto.exe"

# Download and unpack in temp dir

if (!(Test-Path $TempDir)) {
  New-Item $TempDir -ItemType Directory | Out-Null
}

# curl.exe -Lo $DownloadFile $DownloadUrl
$wc = New-Object Net.Webclient
$wc.downloadFile($DownloadUrl, $DownloadFile)

if ($env:PROTO_DEBUG -eq "true") {
  Expand-Archive -Path $DownloadFile -DestinationPath $TempDir -PassThru
} else {
  Expand-Archive -Path $DownloadFile -DestinationPath $TempDir
}

# Move to bin dir and clean up

if (!(Test-Path $InstallDir)) {
  New-Item $InstallDir -ItemType Directory | Out-Null
}

Copy-Item "${TempDir}\proto.exe" -Destination $BinPath -Force
Remove-Item $TempDir -Recurse -Force
Remove-Item $DownloadFile -Force

# Run setup script to update shells

$env:PROTO_LOG = "error"
& $BinPath @('setup')

Write-Output "Successfully installed proto to ${BinPath}"
Write-Output "Launch a new terminal window to start using proto!"
Write-Output ""
Write-Output "Need help? Join our Discord https://discord.gg/qCh9MEynv2"

if ($env:PROTO_DEBUG -eq "true") {
	Write-Output ""
	Write-Output "target=${Target}"
	Write-Output "download_url=${DownloadUrl}"
	Write-Output "bin_path=${BinPath}"
}
