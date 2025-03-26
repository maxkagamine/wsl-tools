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
WizardStyle=classic

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "ja"; MessagesFile: "compiler:Languages\Japanese.isl"

[CustomMessages]
en.AddToPath=Add to PATH
ja.AddToPath=PATHに追加する

[Files]
Source: "dist\wsl-tools\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Tasks]
Name: "AddToPath"; Description: "{cm:AddToPath}"

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

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if (CurStep = ssPostInstall) and WizardIsTaskSelected('AddToPath') then
    AddToPath(ExpandConstant('{app}'));
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
    RemoveFromPath(ExpandConstant('{app}'));
end;
