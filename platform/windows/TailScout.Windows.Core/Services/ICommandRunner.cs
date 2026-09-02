namespace TailScout.Windows.Core.Services;

public interface ICommandRunner
{
    Task<string> RunAsync(IReadOnlyList<string> arguments, CancellationToken cancellationToken = default);
}
