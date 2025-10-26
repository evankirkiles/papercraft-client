export default function Stats() {
  return (
    <div
      className="absolute bottom-4 left-4 bg-card border border-accent rounded-lg p-2"
      aria-label="Toolbar"
    >
      <div className="text-xs text-muted-foreground">
        <p>Paperarium v0.0.1</p>
        <p>{new Date().toLocaleString()}</p>
      </div>
    </div>
  );
}
