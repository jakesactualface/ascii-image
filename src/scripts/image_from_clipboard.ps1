$stream = New-Object System.IO.MemoryStream
$img = Get-Clipboard -Format Image
$img.save($stream, [System.Drawing.Imaging.ImageFormat]::Png)
[System.Convert]::ToBase64String($stream.ToArray())
