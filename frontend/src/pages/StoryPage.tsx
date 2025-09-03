import { useState, useEffect, useRef } from 'react';
import { useParams, Link } from 'react-router-dom';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Slider } from '@/components/ui/slider';
import { Badge } from '@/components/ui/badge';
import { Skeleton } from '@/components/ui/skeleton';
import { ThemeToggle } from '@/components/theme-toggle';
import { ArrowLeft, Mic, Settings, Play, ExternalLink, Globe, LinkIcon, Calendar, AlertCircle, Brain, Loader2, MessageSquare } from 'lucide-react';
import { useToast } from '@/hooks/use-toast';

interface HNStory {
  id: number;
  title: string;
  url?: string;
  text?: string;
  score: number;
  by: string;
  time: number;
  descendants?: number;
  kids?: number[];
}

interface HNComment {
  id: number;
  by?: string;
  time?: number;
  text?: string;
  kids?: number[];
  parent?: number;
}

interface WebsiteMetadata {
  url: string;
  title?: string;
  description?: string;
  domain: string;
  favicon?: string;
}

const StoryPage = () => {
  const { id } = useParams<{ id: string }>();
  const { toast } = useToast();
  const [story, setStory] = useState<HNStory | null>(null);
  const [loading, setLoading] = useState(true);
  const [metadata, setMetadata] = useState<WebsiteMetadata | null>(null);
  const [metadataLoading, setMetadataLoading] = useState(false);
  const [comments, setComments] = useState<HNComment[]>([]);
  const [commentsLoading, setCommentsLoading] = useState(false);
  const [contentGenerating, setContentGenerating] = useState(false);
  const [isGenerating, setIsGenerating] = useState(false);
  
  // Add ref for scrolling
  const loaderRef = useRef<HTMLDivElement>(null);
  
  // Podcast generation state
  const [podcastGenerating, setPodcastGenerating] = useState(false);
  const [podcastThinking, setPodcastThinking] = useState<string[]>([]);
  const [podcastResult, setPodcastResult] = useState<string | null>(null);
  
  // Podcast settings
  const [voice, setVoice] = useState('female');
  const [length, setLength] = useState([2]); // Medium by default
  const [mode, setMode] = useState('summarized');

  useEffect(() => {
    const fetchStory = async () => {
      if (!id) return;
      
      try {
        const response = await fetch(`http://localhost:3001/api/stories/${id}`);
        const storyData = await response.json();
        setStory(storyData);
        
        // Fetch metadata if URL exists
        if (storyData.url) {
          setMetadataLoading(true);
          try {
            const metadataResponse = await fetch(
              `http://localhost:3001/api/metadata?url=${encodeURIComponent(storyData.url)}`
            );
            const metadataData = await metadataResponse.json();
            setMetadata(metadataData);
          } catch (error) {
            console.error('Error fetching metadata:', error);
          } finally {
            setMetadataLoading(false);
          }
        }
      } catch (error) {
        console.error('Error fetching story:', error);
      } finally {
        setLoading(false);
      }
    };

    const fetchComments = async () => {
      if (!id) return;
      
      setCommentsLoading(true);
      try {
        const response = await fetch(`http://localhost:3001/api/stories/${id}/comments`);
        const commentsData = await response.json();
        setComments(commentsData);
        
        toast({
          title: "Comments Loaded",
          description: `Fetched ${commentsData.length} comments.`,
        });
      } catch (error) {
        console.error('Error fetching comments:', error);
        toast({
          title: "Error",
          description: "Failed to fetch comments. Please try again.",
          variant: "destructive",
        });
      } finally {
        setCommentsLoading(false);
      }
    };

    fetchStory();
    fetchComments(); // Automatically fetch comments when page loads
  }, [id]);

  const fetchComments = async () => {
    if (!id) return;
    
    setCommentsLoading(true);
    try {
      const response = await fetch(`http://localhost:3001/api/stories/${id}/comments`);
      const commentsData = await response.json();
      setComments(commentsData);
      
      toast({
        title: "Comments Loaded",
        description: `Fetched ${commentsData.length} comments for content generation.`,
      });
    } catch (error) {
      console.error('Error fetching comments:', error);
      toast({
        title: "Error",
        description: "Failed to fetch comments. Please try again.",
        variant: "destructive",
      });
    } finally {
      setCommentsLoading(false);
    }
  };

  const generateContent = async () => {
    if (!id) return;

    setContentGenerating(true);
    try {
      let commentsToUse = comments;
      
      // If no comments are loaded, fetch them first
      if (comments.length === 0) {
        setCommentsLoading(true);
        try {
          const response = await fetch(`http://localhost:3001/api/stories/${id}/comments`);
          const commentsData = await response.json();
          setComments(commentsData);
          commentsToUse = commentsData;
          
          toast({
            title: "Comments Loaded",
            description: `Fetched ${commentsData.length} comments.`,
          });
        } catch (error) {
          console.error('Error fetching comments:', error);
          toast({
            title: "Error",
            description: "Failed to fetch comments. Please try again.",
            variant: "destructive",
          });
          return;
        } finally {
          setCommentsLoading(false);
        }
      }

      // Filter and extract comment text - use ALL comments
      const commentTexts = commentsToUse
        .filter(comment => comment.text)
        .map(comment => comment.text!);

      if (commentTexts.length === 0) {
        toast({
          title: "No Comments Found",
          description: "This story doesn't have any text comments to process.",
          variant: "destructive",
        });
        return;
      }

      // Send comments to Alchemyst AI
      const response = await fetch('http://localhost:3001/api/generate-content', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          story_id: parseInt(id),
          comments: commentTexts,
        }),
      });

      const result = await response.json();

      if (response.ok) {
        toast({
          title: "Content Added to AI Context!",
          description: `${result.message} - Ready for AI queries.`,
        });
      } else {
        throw new Error(result.error || 'Failed to generate content');
      }
    } catch (error) {
      console.error('Error generating content:', error);
      toast({
        title: "Generation Failed",
        description: error instanceof Error ? error.message : "Failed to generate content. Please check your AI API configuration.",
        variant: "destructive",
      });
    } finally {
      setContentGenerating(false);
    }
  };

  const handleGeneratePodcast = async () => {
    if (!story) return;

    setIsGenerating(true);
    setPodcastGenerating(true);
    setPodcastResult(null);
    setPodcastThinking([]);

    // Scroll to loader
    setTimeout(() => {
      loaderRef.current?.scrollIntoView({ behavior: 'smooth' });
    }, 100);

    try {
      const response = await fetch('http://localhost:3001/api/v1/chat/generate/stream', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          chat_history: [
            {
              content: `Create a podcast episode about "${story.title}". Length: ${['Short (2-3 min)', 'Medium (5-7 min)', 'Long (10-15 min)'][length[0] - 1]}, Style: ${mode}, Voice: ${voice}`,
              role: 'user'
            }
          ],
          persona: 'maya',
          scope: 'internal'
        }),
      });

      if (!response.body) {
        throw new Error('No response body');
      }

      const reader = response.body.getReader();
      const decoder = new TextDecoder();

      try {
        while (true) {
          const { done, value } = await reader.read();
          if (done) break;

          const chunk = decoder.decode(value);
          console.log('Raw chunk:', chunk);
          
          // Split by newlines first to handle multiple SSE events
          const lines = chunk.split('\n');
          
          for (const line of lines) {
            if (!line.startsWith('data: ')) continue;
            
            const data = line.slice(6).trim(); // Remove 'data: ' prefix
            if (data === '[DONE]' || data === 'null') continue;

            try {
              const jsonChunk = JSON.parse(data);
              
              // Only handle final response
              if (jsonChunk.type === 'final_response' && jsonChunk.content) {
                let responseContent = typeof jsonChunk.content === 'string' 
                  ? jsonChunk.content 
                  : JSON.stringify(jsonChunk.content);
                
                // Remove surrounding quotes if present
                if (responseContent.startsWith('"') && responseContent.endsWith('"')) {
                  responseContent = responseContent.slice(1, -1);
                }
                
                // Unescape newlines and other escaped characters
                responseContent = responseContent.replace(/\\n/g, '\n')
                  .replace(/\\"/g, '"')
                  .replace(/\\\\/g, '\\');
                
                setPodcastResult(responseContent);
                
                toast({
                  title: "Podcast Generated Successfully!",
                  description: "Your podcast episode is ready.",
                });
              }
            } catch (parseError) {
              console.error('Error parsing JSON chunk:', parseError);
              continue;
            }
          }
        }
      } finally {
        reader.releaseLock();
      }
    } catch (error) {
      console.error('Error generating podcast:', error);
      toast({
        title: "Podcast Generation Failed",
        description: error instanceof Error ? error.message : "Failed to generate podcast. Please try again.",
        variant: "destructive",
      });
      setPodcastGenerating(false);
      setPodcastResult(null);
      setPodcastThinking([]);
    } finally {
      setIsGenerating(false);
    }
  };

  const formatTimeAgo = (timestamp: number) => {
    const now = Date.now() / 1000;
    const diff = now - timestamp;
    const hours = Math.floor(diff / 3600);
    if (hours < 24) {
      return `${hours}h ago`;
    }
    const days = Math.floor(hours / 24);
    return `${days}d ago`;
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-background">
        {/* Theme Toggle */}
        <div className="fixed top-4 right-4 z-10">
          <ThemeToggle />
        </div>
        
        <div className="container mx-auto px-4 py-8">
          <Skeleton className="h-8 w-32 mb-6" />
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
            <Card className="p-6">
              <Skeleton className="h-6 w-48 mb-6" />
              <div className="space-y-6">
                <Skeleton className="h-20 w-full" />
                <Skeleton className="h-20 w-full" />
                <Skeleton className="h-20 w-full" />
                <Skeleton className="h-12 w-full" />
              </div>
            </Card>
            <Card className="p-6">
              <Skeleton className="h-6 w-32 mb-4" />
              <Skeleton className="h-96 w-full" />
            </Card>
          </div>
        </div>
      </div>
    );
  }

  if (!story) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        {/* Theme Toggle */}
        <div className="fixed top-4 right-4 z-10">
          <ThemeToggle />
        </div>
        
        <div className="text-center">
          <h1 className="text-2xl font-bold text-foreground mb-4">Story not found</h1>
          <Link to="/">
            <Button variant="outline">
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back to Stories
            </Button>
          </Link>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background">
      {/* Theme Toggle */}
      <div className="fixed top-4 right-4 z-10">
        <ThemeToggle />
      </div>
      
      <div className="container mx-auto px-4 py-8">
        {/* Header */}
        <div className="mb-8">
          <Link to="/">
            <Button variant="outline" className="mb-6">
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back to Stories
            </Button>
          </Link>
          
          <div className="mb-4">
            <div className="flex items-center gap-3 mb-3">
              <Badge variant="secondary">HN Story #{story.id}</Badge>
              <span className="text-sm text-muted-foreground">
                {formatTimeAgo(story.time)} • by {story.by}
              </span>
            </div>
            <h1 className="text-3xl font-bold text-foreground leading-tight">
              {story.title}
            </h1>
            <div className="flex items-center gap-4 mt-3 text-sm text-muted-foreground">
              <span><b>HackerNews Points : {story.score} points</b></span>
            </div>
          </div>
        </div>

        {/* Main Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
          {/* Left Column - Podcast Settings */}
          <Card className="p-6">
            <div className="flex items-center gap-2 mb-6">
              <Settings className="w-5 h-5 text-podcast-orange" />
              <h2 className="text-xl font-semibold text-foreground">
                Podcast Settings
              </h2>
            </div>

            <div className="space-y-6">
              {/* Voice Selection */}
              <div>
                <label className="text-sm font-medium text-foreground mb-2 block">
                  Voice
                </label>
                <Select value={voice} onValueChange={setVoice}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="male">Male</SelectItem>
                    <SelectItem value="female">Female</SelectItem>
                    <SelectItem value="ai-persona">AI Persona</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              {/* Podcast Length */}
              <div>
                <label className="text-sm font-medium text-foreground mb-2 block">
                  Podcast Length: {['Short (2-3 min)', 'Medium (5-7 min)', 'Long (10-15 min)'][length[0] - 1]}
                </label>
                <Slider
                  value={length}
                  onValueChange={setLength}
                  max={3}
                  min={1}
                  step={1}
                  className="w-full border-2 border-gray-300"
                />
                <div className="flex justify-between text-xs text-muted-foreground mt-1">
                  <span>Short</span>
                  <span>Medium</span>
                  <span>Long</span>
                </div>
              </div>

              {/* Mode Selection */}
              <div>
                <label className="text-sm font-medium text-foreground mb-2 block">
                  Mode
                </label>
                <Select value={mode} onValueChange={setMode}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="summarized">Summarized</SelectItem>
                    <SelectItem value="all-comments">All Comments</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              {/* Generate Buttons */}
              <div className="items-center justify-center flex gap-3">

                <Button 
                  onClick={generateContent}
                  className="w-full bg-podcast-orange hover:bg-podcast-orange/90 text-podcast-orange-foreground"
                  size="lg"
                  variant="default"
                >
                  {contentGenerating ? (
                    <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  ) : commentsLoading ? (
                    <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  ) : (
                    <Brain className="w-4 h-4 mr-2" />
                  )}
                  {contentGenerating 
                    ? 'Generating Content...' 
                    : commentsLoading 
                    ? 'Loading Comments...'
                    : comments.length > 0 
                    ? `Generate AI Context (${comments.length} comments)`
                    : 'Load & Generate AI Context'
                  }
                </Button>

                
                <Button 
                  onClick={handleGeneratePodcast}
                  disabled={isGenerating}
                  className="w-full bg-podcast-orange hover:bg-podcast-orange/90 text-podcast-orange-foreground"
                  size="lg"
                  variant='outline'
                >
                  {isGenerating ? (
                    <>
                      <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                      Generating...
                    </>
                  ) : (
                    <>
                      <Play className="w-4 h-4 mr-2" />
                      Generate Podcast
                    </>
                  )}
                </Button>
                
              </div>
            </div>
          </Card>

          {/* Comments Section */}
          <Card className="p-6">
            <div className="flex items-center gap-2 mb-4">
              <MessageSquare className="w-5 h-5 text-tech-blue" />
              <h2 className="text-xl font-semibold text-foreground">
                Comments ({comments.length})
              </h2>
            </div>

            {commentsLoading ? (
              <div className="space-y-3">
                {Array.from({ length: 5 }).map((_, i) => (
                  <div key={i} className="space-y-2">
                    <Skeleton className="h-4 w-1/4" />
                    <Skeleton className="h-3 w-full" />
                    <Skeleton className="h-3 w-3/4" />
                  </div>
                ))}
              </div>
            ) : comments.length > 0 ? (
              <div className="space-y-4 max-h-96 overflow-y-auto">
                {comments.map((comment) => (
                  <div key={comment.id} className="border border-border rounded-lg p-4 bg-card">
                    <div className="flex items-center gap-2 mb-2">
                      <span className="text-sm font-medium text-foreground">
                        {comment.by || 'Anonymous'}
                      </span>
                      {comment.time && (
                        <span className="text-xs text-muted-foreground">
                          {formatTimeAgo(comment.time)}
                        </span>
                      )}
                    </div>
                    <div className="text-sm text-foreground leading-relaxed">
                      {comment.text || 'No text content'}
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-center py-8 text-muted-foreground">
                <MessageSquare className="w-12 h-12 mx-auto mb-4 opacity-50" />
                <p>No comments available for this story.</p>
              </div>
            )}
          </Card>

          {/* Right Column - Website Viewer */}
          <Card className="p-6">
            <div className="flex items-center gap-2 mb-4">
              <Globe className="w-5 h-5 text-tech-blue" />
              <h2 className="text-xl font-semibold text-foreground">
                Website Preview
              </h2>
            </div>

            {story?.url ? (
              <div className="space-y-4">
                {/* Website Link Card */}
                <div className="border border-border rounded-lg p-4 bg-card hover:bg-accent/50 transition-colors">
                  <div className="flex items-start gap-3">
                    {/* Favicon */}
                    <div className="w-8 h-8 rounded bg-muted flex items-center justify-center flex-shrink-0 mt-1">
                      {metadataLoading ? (
                        <Skeleton className="w-4 h-4 rounded" />
                      ) : metadata?.favicon ? (
                        <img 
                          src={metadata.favicon} 
                          alt=""
                          className="w-4 h-4 rounded"
                          onError={(e) => {
                            e.currentTarget.style.display = 'none';
                          }}
                        />
                      ) : (
                        <LinkIcon className="w-4 h-4 text-muted-foreground" />
                      )}
                    </div>

                    {/* Content */}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-start justify-between gap-2">
                        <div className="flex-1 min-w-0">
                          {metadataLoading ? (
                            <div className="space-y-2">
                              <Skeleton className="h-5 w-3/4" />
                              <Skeleton className="h-4 w-1/2" />
                              <Skeleton className="h-3 w-full" />
                              <Skeleton className="h-3 w-2/3" />
                            </div>
                          ) : (
                            <>
                              <h3 className="font-medium text-foreground leading-tight mb-1 truncate">
                                {metadata?.title || story.title}
                              </h3>
                              <p className="text-sm text-muted-foreground mb-2">
                                {metadata?.domain || new URL(story.url).hostname}
                              </p>
                              {metadata?.description && (
                                <p className="text-sm text-muted-foreground line-clamp-2 leading-relaxed">
                                  {metadata.description}
                                </p>
                              )}
                            </>
                          )}
                        </div>
                      </div>
                    </div>
                  </div>

                  {/* URL and Action */}
                  <div className="mt-3 pt-3 border-t border-border">
                    <div className="flex items-center justify-between">
                      <div className="text-xs text-muted-foreground font-sans truncate flex-1 mr-2">
                        {story.url}
                      </div>
                      <Button 
                        size="sm"
                        variant="outline"
                        onClick={() => window.open(story.url, '_blank')}
                        className="flex-shrink-0"
                      >
                        <ExternalLink className="w-3 h-3 mr-1" />
                        Visit
                      </Button>
                    </div>
                  </div>
                </div>
              </div>
            ) : (
              <div className="flex items-center justify-center h-96 text-muted-foreground">
                <div className="text-center">
                  <LinkIcon className="w-12 h-12 mb-4 mx-auto opacity-50" />
                  <p className="text-lg font-medium mb-2">No URL Available</p>
                  <p className="text-sm">This story doesn't have an external link to preview.</p>
                </div>
              </div>
            )}
          </Card>
        </div>

        {/* Podcast Content Section */}
        <div className="w-[75vw] mx-auto mt-8">
          {podcastGenerating && !podcastResult && (
            <Card className="p-6" ref={loaderRef}>
              <div className="text-center">
                <div className="flex items-center justify-center mb-4">
                  <Loader2 className="w-12 h-12 animate-spin text-podcast-orange" />
                </div>
                <h2 className="text-2xl font-bold text-foreground mb-2">Generating Podcast</h2>
                <p className="text-muted-foreground">Creating your personalized podcast episode...</p>
              </div>
            </Card>
          )}

          {podcastResult && (
            <Card className="p-6">
              <div className="flex items-center gap-2 mb-4">
                <Play className="w-6 h-6 text-podcast-orange" />
                <h2 className="text-2xl font-bold text-foreground">Generated Podcast Script</h2>
              </div>
              <div className="bg-muted/30 rounded-lg p-4 max-h-[600px] overflow-y-auto">
                <pre className="text-sm text-foreground whitespace-pre-wrap font-mono">
                  {podcastResult}
                </pre>
              </div>
              <div className="flex justify-end mt-4">
                <Button 
                  onClick={() => {
                    setPodcastResult(null);
                    setPodcastGenerating(false);
                    setPodcastThinking([]);
                  }}
                  variant="outline"
                >
                  Clear
                </Button>
              </div>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
};

export default StoryPage;