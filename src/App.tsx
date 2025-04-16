import "./App.css";
import { ThemeProvider } from "@/components/theme-provider";
import { Toaster } from "@/components/ui/sonner";
import { Button } from "./components/ui/button";

function App() {
  return (
    <ThemeProvider defaultTheme="light" storageKey="vite-ui-theme">
      <main className="flex h-screen max-h-screen w-screen items-center justify-center overflow-auto p-2">
        <Button size="lg">CROP TUDO BICHO!</Button>
      </main>
      <Toaster closeButton richColors />
    </ThemeProvider>
  );
}

export default App;
