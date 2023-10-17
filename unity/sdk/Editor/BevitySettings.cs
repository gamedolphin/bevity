using UnityEngine;
using UnityEditor;

public class BevitySettings : ScriptableObject
{
    public string CargoToml;


    private static BevitySettings instance;
    public static BevitySettings Instance
    {
        get
        {
            if (instance == null)
            {
                instance = EnsureBevitySettings();
            }

            return instance;
        }
    }

    private static BevitySettings EnsureBevitySettings()
    {
        var settings = AssetDatabase.LoadAssetAtPath<BevitySettings>("Assets/BevitySettings.asset");
        if (settings != null)
        {
            return settings;
        }

        var created = BevitySettings.CreateInstance<BevitySettings>();
        created.CargoToml = "../rusty/Cargo.toml";

        AssetDatabase.CreateAsset(created, "Assets/BevitySettings.asset");

        return created;
    }
}
