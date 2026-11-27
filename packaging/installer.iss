; Repose-Setup.exe — thin single-file wrapper (kinoz-style).
; It never shows an Inno wizard: it unpacks repose.exe to a temp folder
; and starts Repose's OWN custom setup window (repose.exe --installer).

#define AppName "Repose"
#define AppVer "1.0.0"

[Setup]
AppId={{8F3B4A12-7E91-4C05-B2D8-9E4F1A2C3B01}
AppName={#AppName}
AppVersion={#AppVer}
AppPublisher=P4inz Interactive Labs
AppPublisherURL=https://github.com/p4inz-code/repose-vault
AppCopyright=Copyright (C) 2026 Atharva Patil, P4inz Interactive Labs. Free software under the GNU GPL v3.
VersionInfoVersion={#AppVer}
CreateAppDir=no
Uninstallable=no
DisableStartupPrompt=yes
DisableProgramGroupPage=yes
DisableDirPage=yes
DisableReadyPage=yes
DisableWelcomePage=yes
DisableFinishedPage=yes
OutputDir=..\dist-release
OutputBaseFilename=Repose-Setup-{#AppVer}
SetupIconFile=..\src-tauri\icons\icon.ico
Compression=lzma2/ultra64
SolidCompression=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

[Files]
Source: "..\dist-release\payload.zip"; DestDir: "{tmp}"; Flags: dontcopy

[Code]
function InitializeSetup(): Boolean;
var
  Rc: Integer;
  Dir, Zip, Args: String;
begin
  Result := False;
  ExtractTemporaryFile('payload.zip');
  Zip := ExpandConstant('{tmp}\payload.zip');
  Dir := ExpandConstant('{tmp}\repose-setup');
  ForceDirectories(Dir);
  if (not Exec(ExpandConstant('{sys}\tar.exe'), '-xf "' + Zip + '" -C "' + Dir + '"', '', SW_HIDE, ewWaitUntilTerminated, Rc)) or (Rc <> 0) then
  begin
    MsgBox('Repose could not unpack its setup files. Your PC was not changed.', mbError, MB_OK);
    Exit;
  end;
  Args := '--installer';
  if WizardSilent then Args := Args + ' --silent';
  if not Exec(Dir + '\repose.exe', Args, Dir, SW_SHOW, ewWaitUntilTerminated, Rc) then
    MsgBox('Repose setup could not start. Your PC was not changed.', mbError, MB_OK);
end;
