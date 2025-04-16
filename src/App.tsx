import "./App.css";
import { ThemeProvider } from "@/components/theme-provider";
import { Toaster } from "@/components/ui/sonner";

function App() {
  return (
    <ThemeProvider defaultTheme="light" storageKey="vite-ui-theme">
      <main className="flex h-screen max-h-screen w-screen items-center justify-center overflow-auto p-2">
        {}
      </main>
      <Toaster closeButton richColors />
    </ThemeProvider>
  );
}

export default App;
