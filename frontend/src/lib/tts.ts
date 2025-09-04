export async function downloadWavFromTextGemini(text: string) {
   try {
      console.log('Requesting TTS for text length:', text.length);
      
      const response = await fetch('http://localhost:3001/api/tts/generate', {
         method: 'POST',
         headers: {
            'Content-Type': 'application/json',
         },
         body: JSON.stringify({ text })
      });

      if (!response.ok) {
         const errorData = await response.json().catch(() => ({ error: 'Unknown error' }));
         throw new Error(errorData.error || `HTTP ${response.status}`);
      }

      console.log('Got successful response, getting blob...');
      const blob = await response.blob();
      console.log('Got blob, size:', blob.size);

      // Create download link
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'podcast-audio.wav';
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      console.log('Download initiated');
   } catch (error) {
      console.error('TTS Error:', error);
      throw error;
   }
}