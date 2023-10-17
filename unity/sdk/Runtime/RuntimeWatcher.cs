using UnityEngine;

public class RuntimeWatcher : MonoBehaviour
{
    public System.Action onUpdate;

    private void Update()
    {
        onUpdate?.Invoke();
    }
}
