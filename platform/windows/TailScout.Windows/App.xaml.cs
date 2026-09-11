using Microsoft.UI.Xaml;

namespace TailScout.Windows;

public partial class App : Application
{
    private Window? window;

    public App()
    {
        InitializeComponent();
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        // Exercise the published XAML/resources without touching a user's tailnet
        // or activating a window. Windows CI can run this with redirected output.
        if (Environment.GetCommandLineArgs().Skip(1).SequenceEqual(new[] { "--startup-check" }))
        {
            try
            {
                window = new MainWindow(skipStartupRefresh: true);
                if (window.Content is not FrameworkElement)
                    throw new InvalidOperationException("The main window has no UI content.");
                Console.WriteLine("TailScout startup check passed");
                Exit();
            }
            catch (Exception exception)
            {
                Console.Error.WriteLine(exception);
                Environment.Exit(1);
            }
            return;
        }
        window = new MainWindow();
        window.Activate();
    }
}
