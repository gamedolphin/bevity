using System.Diagnostics;
using UnityEngine;
using UnityEditor;
using System.IO;

public static class Cargo
{
    private const string runCmd = "cargo";
    private const string args = "run --features bevy/dynamic_linking";
    private static Process process;

    public static bool running = false;

    public static void Start(string scenePath)
    {
        process = new Process();
        process.StartInfo.FileName = runCmd;
        process.StartInfo.Environment.Add("ENABLE_BEVITY_EDITOR", "");
        process.StartInfo.Environment.Add("BEVITY_EDITOR_SCENE", scenePath);
        process.StartInfo.Environment.Add("BEVITY_EDITOR_SCENE_GUID", AssetDatabase.AssetPathToGUID(scenePath));
        process.StartInfo.Environment.Add("UNITY_ASSETS_PATH", Path.GetFullPath(Application.dataPath));
        process.StartInfo.Arguments = $"{args}";

        var workingDirectory = Path.Combine(Application.dataPath, BevitySettings.Instance.CargoToml);

        process.StartInfo.WorkingDirectory = Path.GetDirectoryName(workingDirectory);

        // Redirect the output stream of the child process.
        process.StartInfo.RedirectStandardOutput = true;
        process.StartInfo.RedirectStandardError = true;
        process.StartInfo.RedirectStandardInput = true;
        process.StartInfo.UseShellExecute = false;

        UnityEngine.Debug.Log($"Running command: {runCmd} {args} in {process.StartInfo.WorkingDirectory}");

        process.ErrorDataReceived += (sender, data) =>
        {
            if (!string.IsNullOrEmpty(data.Data))
                UnityEngine.Debug.LogError(data.Data);
        };

        process.OutputDataReceived += (sender, data) =>
        {
            if (!string.IsNullOrEmpty(data.Data))
            {
                if (data.Data.StartsWith("EDITOR_CHANGE|"))
                {
                    Watcher.IncomingChange(data.Data);
                }
                else
                {
                    UnityEngine.Debug.Log($"Incoming log: {data.Data}");
                }
            }
        };

        process.Start();

        // Start reading from the streams
        process.BeginOutputReadLine();
        process.BeginErrorReadLine();

        process.EnableRaisingEvents = true;
        process.Exited += new System.EventHandler(OnExitExternal);

        Watcher.Start(process.StandardInput);

        running = true;
        UnityEngine.Physics.autoSimulation = false;
    }

    public static void Stop()
    {
        if (process != null && !process.HasExited)
        {
            process.Kill();
            process.WaitForExit();
            process = null;
        }

        running = false;
        Watcher.Stop();
    }

    private static void OnExitExternal(object sender, System.EventArgs e)
    {
        Stop();
    }
}
