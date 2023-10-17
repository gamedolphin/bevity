using UnityEditor;
using UnityEditor.SceneManagement;
using System.IO;

[InitializeOnLoadAttribute]
public static class PlayMode
{
    static PlayMode()
    {
        EditorApplication.playModeStateChanged += OnPlayModeChange;
    }

    public static void OnPlayModeChange(PlayModeStateChange state)
    {
        UnityEngine.Debug.Log($"Playmode : {state}");
        switch (state)
        {
            case PlayModeStateChange.EnteredPlayMode:
                LaunchBevy();
                break;
            case PlayModeStateChange.ExitingPlayMode:
                ShutdownBevy();
                break;
        }
    }

    private static void LaunchBevy()
    {
        EditorApplication.update += OnUpdate;

        var scene = EditorSceneManager.GetActiveScene();
        var scenePath = Path.GetFullPath(scene.path);

        Cargo.Start(scenePath);
    }

    private static void ShutdownBevy()
    {
        EditorApplication.update -= OnUpdate;
        Cargo.Stop();
    }

    private static void OnUpdate()
    {
        if (!Cargo.running)
        {
            EditorApplication.isPlaying = false;
        }
    }
}
