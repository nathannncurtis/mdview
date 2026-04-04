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
WizardStyle=modern

[Tasks]
Name: "associate"; Description: "Associate .md and .markdown files with mdview"; GroupDescription: "File associations:"

[Files]
Source: "target\release\mdview.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\mdview"; Filename: "{app}\mdview.exe"

[Registry]
; File association (only if task selected)
Root: HKA; Subkey: "Software\Classes\.md"; ValueType: string; ValueData: "mdview"; Flags: uninsdeletevalue; Tasks: associate
Root: HKA; Subkey: "Software\Classes\.markdown"; ValueType: string; ValueData: "mdview"; Flags: uninsdeletevalue; Tasks: associate
Root: HKA; Subkey: "Software\Classes\mdview"; ValueType: string; ValueData: "Markdown File"; Flags: uninsdeletekey; Tasks: associate
Root: HKA; Subkey: "Software\Classes\mdview\DefaultIcon"; ValueType: string; ValueData: "{app}\mdview.exe,0"; Tasks: associate
Root: HKA; Subkey: "Software\Classes\mdview\shell\open\command"; ValueType: string; ValueData: """{app}\mdview.exe"" ""%1"""; Tasks: associate

[Code]
function IsDarkMode: Boolean;
var
  RegValue: Cardinal;
begin
  Result := False;
  if RegQueryDWordValue(HKEY_CURRENT_USER, 'Software\Microsoft\Windows\CurrentVersion\Themes\Personalize', 'AppsUseLightTheme', RegValue) then
    Result := (RegValue = 0);
end;

procedure InitializeWizard;
begin
  if IsDarkMode then
  begin
    WizardForm.Color := $171110;
    WizardForm.MainPanel.Color := $221916;
    WizardForm.InnerPage.Color := $171110;
    WizardForm.WelcomeLabel1.Font.Color := $D9D1C9;
    WizardForm.WelcomeLabel2.Font.Color := $D9D1C9;
    WizardForm.PageNameLabel.Font.Color := $D9D1C9;
    WizardForm.PageDescriptionLabel.Font.Color := $9E948B;
    WizardForm.FilenameLabel.Font.Color := $D9D1C9;
    WizardForm.StatusLabel.Font.Color := $D9D1C9;
    WizardForm.SelectDirBrowseLabel.Font.Color := $D9D1C9;
    WizardForm.DirEdit.Color := $221916;
    WizardForm.DirEdit.Font.Color := $D9D1C9;
    WizardForm.TasksList.Color := $221916;
    WizardForm.TasksList.Font.Color := $D9D1C9;
    WizardForm.ReadyMemo.Color := $221916;
    WizardForm.ReadyMemo.Font.Color := $D9D1C9;
    WizardForm.FinishedHeadingLabel.Font.Color := $D9D1C9;
    WizardForm.FinishedLabel.Font.Color := $D9D1C9;
  end;
end;
