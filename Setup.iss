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
en.MakeVSCodeOpenInWsl=Make launching VS Code from Explorer open in WSL
ja.MakeVSCodeOpenInWsl=エクスプローラーからVS Codeを起動する際にWSLで開くようにする
en.AddRunToShFiles=Add "%1" to context menu of .sh files
ja.AddRunToShFiles=.shファイルのコンテクストメニューに「%1」を追加する
en.RunContextMenuText=Run
ja.RunContextMenuText=実行

[Files]
Source: "dist\wsl-tools\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Tasks]
Name: "AddToPath"; Description: "{cm:AddToPath}"
Name: "AddRunToShFiles"; Description: "{cm:AddRunToShFiles,{cm:RunContextMenuText}}"; Flags: unchecked
Name: "MakeVSCodeOpenInWsl"; Description: "{cm:MakeVSCodeOpenInWsl}"; Flags: unchecked; Check: IsVSCodeInstalled

[Registry]
#define SFA "SOFTWARE\Classes\SystemFileAssociations"
Root: HKLM; Subkey: "{#SFA}\.sh"; Flags: uninsdeletekeyifempty; Tasks: AddRunToShFiles
Root: HKLM; Subkey: "{#SFA}\.sh\shell"; Flags: uninsdeletekeyifempty; Tasks: AddRunToShFiles
Root: HKLM; Subkey: "{#SFA}\.sh\shell\run-in-wsl"; ValueType: string; ValueName: ""; ValueData: "{cm:RunContextMenuText}"; Flags: uninsdeletekey; Tasks: AddRunToShFiles
Root: HKLM; Subkey: "{#SFA}\.sh\shell\run-in-wsl\command"; ValueType: string; ValueName: ""; ValueData: """{app}\run-in-wsl.exe"" ""%1"""; Tasks: AddRunToShFiles

[Code]
const EnvironmentKey = 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment';

function Max(A, B: Integer): Integer;
begin
  if A > B then
    Result := A
  else
    Result := B;
end;

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
  Delete(PathArray, Max(1, P - 1), Length(PathToRemove) + 1)
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
  Result := (GetVSCodeExe() <> '');
end;

{ Inno doesn't delete registry keys when a previously-selected task is deselected (only on uninstall), so "unchecking"
  the task by moving it to Deselected Tasks will prevent updates from overwriting our hijack without deleting them }
procedure ToggleVSCodeSetupTask(Task: String; Checked: Boolean);
var
  Hive: Integer;
  Key: String;
  FromValue: String;
  ToValue: String;
  FromTasks: String;
  ToTasks: String;
  P: Integer;
begin
  if Checked then begin
    FromValue := 'Inno Setup: Deselected Tasks';
    ToValue := 'Inno Setup: Selected Tasks';
  end else begin
    FromValue := 'Inno Setup: Selected Tasks';
    ToValue := 'Inno Setup: Deselected Tasks';
  end;

  { Find the registry key for VS Code's installer (the "User Installer" and "System Installer" are separate) }
  Hive := HKEY_CURRENT_USER;
  Key := 'SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\{771FD6B0-FA20-440A-A002-3B3BAC16DC50}_is1';
  if not RegKeyExists(Hive, Key) then
  begin
    Hive := HKEY_LOCAL_MACHINE;
    Key := 'SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\{EA457B21-F73E-494C-ACAB-524FDE069978}_is1';
    if not RegKeyExists(Hive, Key) then
      exit;
  end;

  { Remove the task from the "from" list }
  RegQueryStringValue(Hive, Key, FromValue, FromTasks);
  P := Pos(',' + Uppercase(Task) + ',', ',' + Uppercase(FromTasks) + ',');
  if P > 0 then
  begin
    Delete(FromTasks, Max(1, P - 1), Length(Task) + 1);
    RegWriteStringValue(Hive, Key, FromValue, FromTasks);
  end;

  { Add the task to the "to" list }
  RegQueryStringValue(Hive, Key, ToValue, ToTasks);
  if Pos(',' + Uppercase(Task) + ',', ',' + Uppercase(ToTasks) + ',') = 0 then
  begin
    ToTasks := ToTasks + ',' + Task;
    RegWriteStringValue(Hive, Key, ToValue, ToTasks);
  end;
end;

{ https://github.com/microsoft/vscode/blob/1debf21160174ecaf114e8e043146da08ba25d4a/build/win32/code.iss }
procedure UpdateVSCodeRegistryKeys();
var
  Exe: String;
  Enabling: Boolean;
  Keys: array of String;
  I: Integer;
begin
  { Bail if VS Code isn't installed }
  if not IsVSCodeInstalled() then
    exit;

  { Decide if we're enabling our registry hijack or reverting it }
  if (IsUninstaller() and WizardIsTaskSelected('MakeVSCodeOpenInWsl')) or
     not WizardIsTaskSelected('MakeVSCodeOpenInWsl') then
  begin
    Exe := GetVSCodeExe();
    Enabling := False;
  end
  else if WizardIsTaskSelected('MakeVSCodeOpenInWsl') then
  begin
    Exe := ExpandConstant('{app}\code-wsl.exe');
    Enabling := True;
  end
  else
    exit;

  { File context menu }
  if RegKeyExists(HKEY_CLASSES_ROOT, '*\shell\VSCode\command') then
  begin
    ToggleVSCodeSetupTask('addcontextmenufiles', not Enabling);
    RegWriteExpandStringValue(HKEY_CLASSES_ROOT, '*\shell\VSCode\command', '', '"' + Exe + '" "%1"');
  end;

  { Folder context menu }
  if RegKeyExists(HKEY_CLASSES_ROOT, 'Directory\shell\VSCode\command') then
  begin
    ToggleVSCodeSetupTask('addcontextmenufolders', not Enabling);
    RegWriteExpandStringValue(HKEY_CLASSES_ROOT, 'Directory\shell\VSCode\command', '', '"' + Exe + '" "%V"');
    RegWriteExpandStringValue(HKEY_CLASSES_ROOT, 'Directory\Background\shell\VSCode\command', '', '"' + Exe + '" "%V"');
    RegWriteExpandStringValue(HKEY_CLASSES_ROOT, 'Drive\shell\VSCode\command', '', '"' + Exe + '" "%V"');
    { This last one isn't added by VS Code's installer; I added it myself since the libraries feature is handy for
      combining a Projects folder in WSL with one Windows-side. This has no effect unless you've done the same. }
    if RegKeyExists(HKEY_CLASSES_ROOT, 'LibraryFolder\Background\shell\VSCode\command') then
      RegWriteExpandStringValue(HKEY_CLASSES_ROOT, 'LibraryFolder\Background\shell\VSCode\command', '', '"' + Exe + '" "%V"');
  end;

  { File associations }
  if RegKeyExists(HKEY_CLASSES_ROOT, 'VSCode.txt') then
  begin
    ToggleVSCodeSetupTask('associatewithfiles', not Enabling);
    RegGetSubkeyNames(HKEY_CLASSES_ROOT, '', Keys);
    for I := 0 to GetArrayLength(Keys) - 1 do
    begin
      if Pos('VSCode.', Keys[I]) <> 1 then
        continue;
      RegWriteExpandStringValue(HKEY_CLASSES_ROOT, Keys[I] + '\shell\open\command', '', '"' + Exe + '" "%1"');
    end;
  end;
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
