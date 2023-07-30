$stream = New-Object System.IO.MemoryStream
$img = Get-Clipboard -Format Image
if ( $img -eq $null )
{
	$filename = Get-Clipboard -Format FileDropList -Raw
	$bytes = [System.IO.File]::ReadAllBytes($filename)
	$stream = [System.IO.MemoryStream]::new($bytes)
	[System.Convert]::ToBase64String($stream.ToArray())
}
else
{
	$img.save($stream, [System.Drawing.Imaging.ImageFormat]::Png)
	[System.Convert]::ToBase64String($stream.ToArray())
}
