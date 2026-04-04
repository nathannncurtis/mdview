[Setup]
AppName=mdview
AppVersion={#GetEnv('MDVIEW_VERSION')}
DefaultDirName={autopf}\mdview
DefaultGroupName=mdview
OutputBaseFilename=mdview-setup
Compression=lzma2
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
ChangesAssociations=yes
UninstallDisplayIcon={app}\mdview.exe

[Files]
Source: "target\release\mdview.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\mdview"; Filename: "{app}\mdview.exe"

[Registry]
Root: HKCU; Subkey: "Software\Classes\.md"; ValueType: string; ValueData: "mdview"; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.markdown"; ValueType: string; ValueData: "mdview"; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\mdview"; ValueType: string; ValueData: "Markdown File"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\mdview\DefaultIcon"; ValueType: string; ValueData: "{app}\mdview.exe,0"
Root: HKCU; Subkey: "Software\Classes\mdview\shell\open\command"; ValueType: string; ValueData: """{app}\mdview.exe"" ""%1"""
