using System.Collections.Generic;
using UnityEngine;
using UnityEditor;
using UnityEditor.SceneManagement;
using System.IO;
using System.Diagnostics;
using System.Text;

[InitializeOnLoadAttribute]
public static class Initializer
{
    static Initializer()
    {
        var settings = AssetDatabase.LoadAssetAtPath<BevitySettings>("Assets/BevitySettings.asset");
        if (settings == null)
        {
            var created = BevitySettings.CreateInstance<BevitySettings>();
            created.CargoToml = "../rusty/Cargo.toml";

            AssetDatabase.CreateAsset(created, "Assets/BevitySettings.asset");
        }
    }
}
