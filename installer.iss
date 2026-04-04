[Setup]
AppName=mdview
AppVersion={#MDVIEW_VERSION}
AppPublisher=nathannncurtis
AppPublisherURL=https://github.com/nathannncurtis/mdview
DefaultDirName={autopf}\mdview
DefaultGroupName=mdview
OutputBaseFilename=mdview-setup
Compression=lzma2
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
ChangesAssociations=yes
UninstallDisplayIcon={app}\mdview.exe
PrivilegesRequiredOverridesAllowed=dialog
PrivilegesRequired=lowest
WizardStyle=dynamic

[Tasks]
Name: "associate"; Description: "Associate .md and .markdown files with mdview"; GroupDescription: "File associations:"

[Files]
Source: "target\release\mdview.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\mdview"; Filename: "{app}\mdview.exe"

[Registry]
Root: HKA; Subkey: "Software\Classes\.md"; ValueType: string; ValueData: "mdview"; Flags: uninsdeletevalue; Tasks: associate
Root: HKA; Subkey: "Software\Classes\.markdown"; ValueType: string; ValueData: "mdview"; Flags: uninsdeletevalue; Tasks: associate
Root: HKA; Subkey: "Software\Classes\mdview"; ValueType: string; ValueData: "Markdown File"; Flags: uninsdeletekey; Tasks: associate
Root: HKA; Subkey: "Software\Classes\mdview\DefaultIcon"; ValueType: string; ValueData: "{app}\mdview.exe,0"; Tasks: associate
Root: HKA; Subkey: "Software\Classes\mdview\shell\open\command"; ValueType: string; ValueData: """{app}\mdview.exe"" ""%1"""; Tasks: associate
