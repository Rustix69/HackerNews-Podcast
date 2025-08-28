# HackerNews Podcast Generator

A full-stack application that transforms HackerNews stories into podcast content with configurable settings.

## Architecture

### Backend (Rust)
- **Framework**: Axum web framework
- **Functionality**: 
  - Fetches top stories from HackerNews API
  - Provides REST API endpoints for frontend
  - Handles CORS for cross-origin requests
  - Concurrent API calls for better performance

### Frontend (React + TypeScript)
- **Framework**: React with TypeScript and Vite
- **UI**: Shadcn/ui components with Tailwind CSS
- **Features**:
  - Browse top HackerNews stories
  - Configure podcast settings (voice, length, mode)
  - Responsive design with dark/light theme

## API Endpoints

### Backend (Port 3001)
- `GET /health` - Health check
- `GET /api/stories` - Get top 50 stories from HackerNews
- `GET /api/stories/:id` - Get specific story by ID

### Frontend (Port 5173)
- Main application accessible via web browser

## Running the Application

### Start Backend
```bash
cd backend
cargo run
```
The backend will start on `http://localhost:3001`

### Start Frontend
```bash
cd frontend
npm install
npm run dev
```
The frontend will start on `http://localhost:5173`

## Dependencies

### Backend (Rust)
- `axum` - Web framework
- `tokio` - Async runtime
- `reqwest` - HTTP client for HackerNews API
- `serde` - JSON serialization
- `tower-http` - CORS middleware
- `futures` - Concurrent request handling

### Frontend (React)
- `react` & `react-dom` - Core React
- `react-router-dom` - Routing
- `tailwindcss` - Styling
- `lucide-react` - Icons
- `@radix-ui` - UI primitives
- Various shadcn/ui components

## Features

1. **Story Browsing**: View top HackerNews stories with pagination
2. **Story Details**: View individual story information
3. **Podcast Configuration**: 
   - Voice selection (Male/Female/AI Persona)
   - Length settings (Short/Medium/Long)
   - Mode selection (Summarized/All Comments)
4. **Modern UI**: Dark/light theme toggle, responsive design
5. **Performance**: Backend handles API aggregation and caching

## Data Flow

1. Frontend requests stories from backend
2. Backend fetches from HackerNews API concurrently
3. Backend filters and returns processed story data
4. Frontend displays stories with pagination and interactive elements
5. User can configure podcast settings for any story

## Next Steps

- Implement actual podcast generation
- Add audio playback functionality
- Implement caching layer for better performance
- Add user preferences and history 
