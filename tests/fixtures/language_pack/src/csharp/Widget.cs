using System.Collections.Generic;

namespace LanguagePack.CSharp
{
    public interface CSharpRenderable
    {
        void Render();
    }

    public struct CSharpOptions
    {
        public int Limit;
    }

    public enum CSharpMode
    {
        Compact
    }

    public record CSharpRecord(string Name);

    public class CSharpWidget
    {
        private const int CSharpLimit = 4;

        public string Name { get; }

        public CSharpWidget(string name)
        {
            Name = name;
        }

        public void Render(
            string message
        )
        {
            var ignored = "class CSharpStringFake {}";
        }
    }
}
