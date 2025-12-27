; Inno docs: https://jrsoftware.org/ishelp/
; Preprocessor docs: https://jrsoftware.org/ispphelp/
#pragma verboselevel 9

; Extract version from one of the exe's
#define Exe "dist\wsl-tools\recycle.exe"
#ifnexist Exe
  #pragma error Exe + " does not exist"
#endif
#define FileVersion GetStringFileInfo(Exe, "FileVersion")
#define ProductVersion GetStringFileInfo(Exe, "ProductVersion")
#define Version Copy(ProductVersion, 1, Pos("+", ProductVersion) - 1)
#pragma message "Version is " + Version

[Setup]
AppCopyright=Copyright © Max Kagamine
AppId={{C5EB6945-50E0-46B8-A7C8-883105DCC39}
AppName=wsl-tools
AppPublisher=Max Kagamine
AppPublisherURL=https://github.com/maxkagamine/wsl-tools
AppSupportURL=https://github.com/maxkagamine/wsl-tools/issues
AppUpdatesURL=https://github.com/maxkagamine/wsl-tools/releases
AppVerName=wsl-tools {#Version}
AppVersion={#Version}
ArchitecturesAllowed=x64os
ArchitecturesInstallIn64BitMode=x64compatible
ChangesEnvironment=yes
DefaultDirName={autopf}\wsl-tools
LicenseFile=LICENSE.txt
MinVersion=6.2
OutputBaseFilename=wsl-tools-installer
OutputDir=dist
SolidCompression=yes
UninstallFilesDir={app}\uninst
VersionInfoProductTextVersion={#ProductVersion}
VersionInfoVersion={#FileVersion}
WizardStyle=classic dynamic

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "ja"; MessagesFile: "compiler:Languages\Japanese.isl"

[CustomMessages]
en.AddToPath=Add to PATH
ja.AddToPath=PATHに追加する
en.MakeOpenWithCodeOpenInWsl=Make "Open with Code" open in WSL
ja.MakeOpenWithCodeOpenInWsl=「Code で開く」をWSLで開くようにする
en.AddRunToShFiles=Add "%1" to context menu of .sh files
ja.AddRunToShFiles=.shファイルのコンテクストメニューに「%1」を追加する
en.RunContextMenuText=Run
ja.RunContextMenuText=実行

[Files]
Source: "dist\wsl-tools\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Tasks]
Name: "AddToPath"; Description: "{cm:AddToPath}"
Name: "MakeOpenWithCodeOpenInWsl"; Description: "{cm:MakeOpenWithCodeOpenInWsl}"; Flags: unchecked; Check: IsVSCodeInstalled
Name: "AddRunToShFiles"; Description: "{cm:AddRunToShFiles,{cm:RunContextMenuText}}"; Flags: unchecked

[Registry]
#define SFA "SOFTWARE\Classes\SystemFileAssociations"
Root: HKLM; Subkey: "{#SFA}\.sh"; Flags: uninsdeletekeyifempty; Tasks: AddRunToShFiles
Root: HKLM; Subkey: "{#SFA}\.sh\shell"; Flags: uninsdeletekeyifempty; Tasks: AddRunToShFiles
Root: HKLM; Subkey: "{#SFA}\.sh\shell\run-in-wsl"; ValueType: string; ValueName: ""; ValueData: "{cm:RunContextMenuText}"; Flags: uninsdeletekey; Tasks: AddRunToShFiles
Root: HKLM; Subkey: "{#SFA}\.sh\shell\run-in-wsl\command"; ValueType: string; ValueName: ""; ValueData: """{app}\run-in-wsl.exe"" ""%1"""; Tasks: AddRunToShFiles

[Code]
const EnvironmentKey = 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment';

procedure AddToPath(PathToAdd: string);
var
  PathArray: string;
begin
  { Get current PATH }
  if not RegQueryStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', PathArray) then
    PathArray := '';

  { Bail if PathToAdd is already present }
  if Pos(';' + Uppercase(PathToAdd) + ';', ';' + Uppercase(PathArray) + ';') > 0 then
    exit;

  { Update PATH }
  PathArray := PathArray + ';' + PathToAdd
  if not RegWriteStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', PathArray) then
    RaiseException('Could not write to HKEY_LOCAL_MACHINE\' + EnvironmentKey);
end;

procedure RemoveFromPath(PathToRemove: string);
var
  PathArray: string;
  P: integer;
begin
  { Get current PATH }
  if not RegQueryStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', PathArray) then
    exit;

  { Bail if PathToRemove is not present }
  P := Pos(';' + Uppercase(PathToRemove) + ';', ';' + Uppercase(PathArray) + ';');
  if P = 0 then
    exit;

  { Update PATH }
  Delete(PathArray, P - 1, Length(PathToRemove) + 1)
  if not RegWriteStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', PathArray) then
    RaiseException('Could not write to HKEY_LOCAL_MACHINE\' + EnvironmentKey);
end;

function GetVSCodeExe(): String;
var
  Command: String;
  Args: array of String;
begin
  if not RegQueryStringValue(HKEY_CLASSES_ROOT, 'Applications\Code.exe\shell\open\command', '', Command) then
  begin
    Result := '';
    exit;
  end;

  Args := StringSplitEx(Command, [' '], '"', stExcludeEmpty);
  Result := RemoveQuotes(Args[0]);
end;

function IsVSCodeInstalled(): Boolean;
begin
  Result := (GetVSCodeExe() <> '') and RegKeyExists(HKEY_CLASSES_ROOT, '*\shell\VSCode\command');
end;

function ShouldResetVSCodeRegistryKeys(): Boolean;
var
  Command: String;
begin
  Result := RegQueryStringValue(HKEY_CLASSES_ROOT, '*\shell\VSCode\command', '', Command) and (Pos('code-wsl.exe', Command) > 0);
end;

procedure UpdateVSCodeRegistryKeys();
var
  Exe: String;
begin
  { Bail if VS Code isn't installed }
  if not IsVSCodeInstalled() then
    exit;

  { Decide if we're replacing Code.exe with our binary, setting it back to the original, or leaving it as is }
  if (IsUninstaller() or not WizardIsTaskSelected('MakeOpenWithCodeOpenInWsl')) and ShouldResetVSCodeRegistryKeys() then
    Exe := GetVSCodeExe()
  else if WizardIsTaskSelected('MakeOpenWithCodeOpenInWsl') then
    Exe := ExpandConstant('{app}\code-wsl.exe')
  else
    exit;

  { Update all the things }
  { https://github.com/microsoft/vscode/blob/50b5aa895467bcc17c91c9d2357f670969d4da3d/build/win32/code.iss#L1270C1-L1280C1 }
  RegWriteExpandStringValue(HKEY_CLASSES_ROOT, '*\shell\VSCode\command', '', '"' + Exe + '" "%1"');
  RegWriteExpandStringValue(HKEY_CLASSES_ROOT, 'Directory\shell\VSCode\command', '', '"' + Exe + '" "%V"');
  RegWriteExpandStringValue(HKEY_CLASSES_ROOT, 'Directory\Background\shell\VSCode\command', '', '"' + Exe + '" "%V"');
  RegWriteExpandStringValue(HKEY_CLASSES_ROOT, 'Drive\shell\VSCode\command', '', '"' + Exe + '" "%V"');

  { "Open with Code" doesn't normally show up when right-clicking the background of a Library, but I added it myself }
  { since the Libraries feature is useful for combining a Projects folder in WSL with a separate one in Windows }
  if RegKeyExists(HKEY_CLASSES_ROOT, 'LibraryFolder\Background\shell\VSCode\command') then
    RegWriteExpandStringValue(HKEY_CLASSES_ROOT, 'LibraryFolder\Background\shell\VSCode\command', '', '"' + Exe + '" "%V"');
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
  begin
    if WizardIsTaskSelected('AddToPath') then
      AddToPath(ExpandConstant('{app}'))
    else
      RemoveFromPath(ExpandConstant('{app}'));

    UpdateVSCodeRegistryKeys();
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    RemoveFromPath(ExpandConstant('{app}'));
    UpdateVSCodeRegistryKeys();
  end;
end;
